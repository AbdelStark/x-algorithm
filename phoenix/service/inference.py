import hashlib
from dataclasses import dataclass
from typing import Dict, Iterable, List, Optional, Sequence, Tuple

import numpy as np

from grok import TransformerConfig
from recsys_model import HashConfig, PhoenixModelConfig, RecsysBatch, RecsysEmbeddings
from runners import ACTIONS, ModelRunner, RecsysInferenceRunner

from .schema import CandidateScore, PhoenixScores, RankingRequest, RankingResponse

VOCAB_SIZE = 100_000
DEFAULT_HISTORY_LEN = 50
DEFAULT_CANDIDATE_LEN = 10
VQV_DURATION_THRESHOLD = 6.0
SCORE_OFFSET = 1.0


@dataclass
class PhoenixRanker:
    history_len: int = DEFAULT_HISTORY_LEN
    candidate_len: int = DEFAULT_CANDIDATE_LEN
    emb_size: int = 128
    _runner: Optional[RecsysInferenceRunner] = None

    def initialize(self) -> None:
        hash_config = HashConfig(num_user_hashes=2, num_item_hashes=2, num_author_hashes=2)
        model = PhoenixModelConfig(
            emb_size=self.emb_size,
            num_actions=len(ACTIONS),
            history_seq_len=self.history_len,
            candidate_seq_len=self.candidate_len,
            hash_config=hash_config,
            product_surface_vocab_size=16,
            model=TransformerConfig(
                emb_size=self.emb_size,
                widening_factor=2,
                key_size=64,
                num_q_heads=2,
                num_kv_heads=2,
                num_layers=2,
                attn_output_multiplier=0.125,
            ),
        )

        runner = RecsysInferenceRunner(
            runner=ModelRunner(model=model, bs_per_device=0.125),
            name="phoenix_service",
        )
        runner.initialize()
        self._runner = runner

    def rank(self, request: RankingRequest) -> RankingResponse:
        if self._runner is None:
            raise RuntimeError("PhoenixRanker is not initialized")

        batch = build_batch(request, self._runner.runner.model)
        embeddings = build_embeddings(batch, self._runner.runner.model.emb_size)
        output = self._runner.rank(batch, embeddings)

        scores = np.array(output.scores[0])
        candidate_count = len(request.candidates)
        primary_scores = scores[:candidate_count, 0]
        ranked_indices = np.argsort(-primary_scores)
        rank_map = {int(idx): rank + 1 for rank, idx in enumerate(ranked_indices)}

        candidate_scores: List[CandidateScore] = []
        for idx, candidate in enumerate(request.candidates[:candidate_count]):
            probs = scores[idx]
            phoenix_scores = probs_to_scores(probs)
            weighted_score = compute_weighted_score(
                phoenix_scores,
                candidate.video_duration_seconds,
            )
            candidate_scores.append(
                CandidateScore(
                    post_id=candidate.post_id,
                    phoenix_scores=phoenix_scores,
                    weighted_score=weighted_score,
                    rank=rank_map.get(idx, candidate_count),
                )
            )

        return RankingResponse(scores=candidate_scores)


def build_batch(request: RankingRequest, model: PhoenixModelConfig) -> RecsysBatch:
    history_len = model.history_seq_len
    candidate_len = model.candidate_seq_len
    num_actions = model.num_actions
    hash_config = model.hash_config

    user_hashes = np.array(
        [multi_hashes(request.user_id, hash_config.num_user_hashes, VOCAB_SIZE)],
        dtype=np.int32,
    )

    history_post_hashes = np.zeros(
        (1, history_len, hash_config.num_item_hashes), dtype=np.int32
    )
    history_author_hashes = np.zeros(
        (1, history_len, hash_config.num_author_hashes), dtype=np.int32
    )
    history_actions = np.zeros((1, history_len, num_actions), dtype=np.float32)
    history_product_surface = np.zeros((1, history_len), dtype=np.int32)

    for idx, post in enumerate(request.history_posts[:history_len]):
        history_post_hashes[0, idx, :] = multi_hashes(
            str(post.text_hash), hash_config.num_item_hashes, VOCAB_SIZE
        )
        history_author_hashes[0, idx, :] = multi_hashes(
            str(post.author_hash), hash_config.num_author_hashes, VOCAB_SIZE
        )
        history_product_surface[0, idx] = post.product_surface

    for idx, actions in enumerate(request.history_actions[:history_len]):
        actions_vec = pad_list(actions, num_actions, 0.0)
        history_actions[0, idx, :] = np.array(actions_vec, dtype=np.float32)

    candidate_post_hashes = np.zeros(
        (1, candidate_len, hash_config.num_item_hashes), dtype=np.int32
    )
    candidate_author_hashes = np.zeros(
        (1, candidate_len, hash_config.num_author_hashes), dtype=np.int32
    )
    candidate_product_surface = np.zeros((1, candidate_len), dtype=np.int32)

    for idx, candidate in enumerate(request.candidates[:candidate_len]):
        candidate_post_hashes[0, idx, :] = multi_hashes(
            str(candidate.text_hash), hash_config.num_item_hashes, VOCAB_SIZE
        )
        candidate_author_hashes[0, idx, :] = multi_hashes(
            str(candidate.author_hash), hash_config.num_author_hashes, VOCAB_SIZE
        )
        candidate_product_surface[0, idx] = candidate.product_surface

    return RecsysBatch(
        user_hashes=user_hashes,
        history_post_hashes=history_post_hashes,
        history_author_hashes=history_author_hashes,
        history_actions=history_actions,
        history_product_surface=history_product_surface,
        candidate_post_hashes=candidate_post_hashes,
        candidate_author_hashes=candidate_author_hashes,
        candidate_product_surface=candidate_product_surface,
    )


def build_embeddings(batch: RecsysBatch, emb_size: int) -> RecsysEmbeddings:
    embedding_table: Dict[int, np.ndarray] = {}

    user_embeddings = hashes_to_embeddings(batch.user_hashes, emb_size, embedding_table)
    history_post_embeddings = hashes_to_embeddings(
        batch.history_post_hashes, emb_size, embedding_table
    )
    history_author_embeddings = hashes_to_embeddings(
        batch.history_author_hashes, emb_size, embedding_table
    )
    candidate_post_embeddings = hashes_to_embeddings(
        batch.candidate_post_hashes, emb_size, embedding_table
    )
    candidate_author_embeddings = hashes_to_embeddings(
        batch.candidate_author_hashes, emb_size, embedding_table
    )

    return RecsysEmbeddings(
        user_embeddings=user_embeddings,
        history_post_embeddings=history_post_embeddings,
        candidate_post_embeddings=candidate_post_embeddings,
        history_author_embeddings=history_author_embeddings,
        candidate_author_embeddings=candidate_author_embeddings,
    )


def hashes_to_embeddings(
    hashes: np.ndarray, emb_size: int, table: Dict[int, np.ndarray]
) -> np.ndarray:
    embeddings = np.zeros(hashes.shape + (emb_size,), dtype=np.float32)
    it = np.nditer(hashes, flags=["multi_index"])
    while not it.finished:
        value = int(it[0])
        if value == 0:
            embedding = np.zeros((emb_size,), dtype=np.float32)
        else:
            embedding = table.get(value)
            if embedding is None:
                rng = np.random.default_rng(value)
                embedding = rng.normal(0.0, 0.2, size=(emb_size,)).astype(np.float32)
                table[value] = embedding
        embeddings[it.multi_index] = embedding
        it.iternext()
    return embeddings


def multi_hashes(value: str, num_hashes: int, vocab_size: int) -> np.ndarray:
    hashes = [stable_hash(value, idx + 1) for idx in range(num_hashes)]
    normalized = [(hash_value % (vocab_size - 1)) + 1 for hash_value in hashes]
    return np.array(normalized, dtype=np.int32)


def stable_hash(value: str, seed: int) -> int:
    payload = f"{seed}:{value}".encode("utf-8")
    digest = hashlib.sha256(payload).digest()
    return int.from_bytes(digest[:8], "big")


def pad_list(values: Sequence[float], length: int, pad_value: float) -> List[float]:
    result = list(values)[:length]
    if len(result) < length:
        result.extend([pad_value] * (length - len(result)))
    return result


def probs_to_scores(probs: Iterable[float]) -> PhoenixScores:
    values = list(probs)
    return PhoenixScores(
        like=float(values[0]),
        reply=float(values[1]),
        repost=float(values[2]),
        photo_expand=float(values[3]),
        click=float(values[4]),
        profile_click=float(values[5]),
        video_view=float(values[6]),
        share=float(values[7]),
        share_dm=float(values[8]),
        share_link=float(values[9]),
        dwell=float(values[10]),
        quote=float(values[11]),
        quoted_click=float(values[12]),
        follow_author=float(values[13]),
        not_interested=float(values[14]),
        block=float(values[15]),
        mute=float(values[16]),
        report=float(values[17]),
        dwell_time=float(values[18]),
    )


def compute_weighted_score(scores: PhoenixScores, video_duration: Optional[float]) -> float:
    total = 0.0
    total += scores.like * 1.0
    total += scores.reply * 1.6
    total += scores.repost * 2.0
    total += scores.photo_expand * 0.3
    total += scores.click * 0.4
    total += scores.profile_click * 0.3
    if video_duration is not None and video_duration >= VQV_DURATION_THRESHOLD:
        total += scores.video_view * 0.5
    total += scores.share * 1.4
    total += scores.share_dm * 0.8
    total += scores.share_link * 0.6
    total += scores.dwell * 0.2
    total += scores.quote * 1.7
    total += scores.quoted_click * 0.5
    total += scores.follow_author * 1.2
    total += scores.not_interested * -2.5
    total += scores.block * -5.0
    total += scores.mute * -3.0
    total += scores.report * -6.0
    total += scores.dwell_time * 0.1

    if total < 0.0:
        total += SCORE_OFFSET

    return total
