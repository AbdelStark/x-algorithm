from typing import List, Optional

from pydantic import BaseModel, Field


class PostFeatures(BaseModel):
    post_id: str
    author_id: str
    text_hash: int
    author_hash: int
    product_surface: int = 0
    video_duration_seconds: Optional[float] = None


class RankingRequest(BaseModel):
    user_id: str
    user_embedding: Optional[List[float]] = None
    history_posts: List[PostFeatures] = Field(default_factory=list)
    history_actions: List[List[float]] = Field(default_factory=list)
    candidates: List[PostFeatures]


class PhoenixScores(BaseModel):
    like: float
    reply: float
    repost: float
    photo_expand: float
    click: float
    profile_click: float
    video_view: float
    share: float
    share_dm: float
    share_link: float
    dwell: float
    quote: float
    quoted_click: float
    follow_author: float
    not_interested: float
    block: float
    mute: float
    report: float
    dwell_time: float


class CandidateScore(BaseModel):
    post_id: str
    phoenix_scores: PhoenixScores
    weighted_score: float
    rank: int


class RankingResponse(BaseModel):
    scores: List[CandidateScore]
