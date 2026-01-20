import logging
import os
from fastapi import FastAPI, HTTPException

from .inference import PhoenixRanker
from .schema import RankingRequest, RankingResponse

logger = logging.getLogger("phoenix.service")


def env_int(name: str, default: int) -> int:
    value = os.getenv(name)
    if value is None:
        return default
    try:
        return int(value)
    except ValueError:
        return default


app = FastAPI(title="Phoenix Ranking Service")
ranker = PhoenixRanker(
    history_len=env_int("PHOENIX_HISTORY_LEN", 50),
    candidate_len=env_int("PHOENIX_CANDIDATE_LEN", 10),
    emb_size=env_int("PHOENIX_EMB_SIZE", 128),
)


@app.on_event("startup")
async def startup_event() -> None:
    logger.info("Initializing Phoenix ranker...")
    ranker.initialize()
    logger.info("Phoenix ranker ready.")


@app.post("/rank", response_model=RankingResponse)
async def rank(request: RankingRequest) -> RankingResponse:
    try:
        return ranker.rank(request)
    except Exception as exc:  # pylint: disable=broad-except
        logger.exception("Phoenix rank failed")
        raise HTTPException(status_code=500, detail=str(exc))
