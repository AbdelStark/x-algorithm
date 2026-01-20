# Calibration Guide

This document describes how to collect calibration samples and run the built-in calibration tooling.

## Calibration Sample Schema

Create a JSON array of samples that match the `CalibrationSample` struct used by the simulator.

```json
[
  {
    "post_id": "123456",
    "post_text": "Shipping a new feature today.",
    "author_followers": 1200,
    "author_following": 350,
    "account_age_days": 900,
    "avg_engagement_rate": 0.02,
    "posts_per_day": 1.4,
    "verified": false,
    "media_type": "none",
    "actual_impressions": 1800,
    "actual_likes": 64,
    "actual_replies": 8,
    "actual_reposts": 6,
    "actual_quotes": 2,
    "actual_shares": 4
  }
]
```

Required fields:
- `post_id`, `post_text`, `author_followers`, `media_type`
- `actual_impressions`, `actual_likes`, `actual_replies`, `actual_reposts`

Optional fields:
- `author_following`, `account_age_days`, `avg_engagement_rate`, `posts_per_day`, `verified`
- `actual_quotes`, `actual_shares`

## Running Calibration

```bash
cargo run -- calibrate --data data/calibration.json
```

The command prints:
- impression correlation
- engagement rate correlation
- MAE for like/reply/repost rates
- pairwise ranking accuracy

To save a JSON report:

```bash
cargo run -- calibrate --data data/calibration.json --report data/calibration_report.json
```

## Weight Tuning

To run a lightweight random-search tuning pass:

```bash
cargo run -- calibrate --data data/calibration.json --tune
```

This prints tuned weights as a TOML block. You can copy them into `config/scoring.toml` under the `[weights]` section.

## Tips

- Use at least 100 samples for stable correlations.
- Normalize media types to `none`, `image`, `gif`, or `video`.
- Include a mix of high and low performing posts to improve ranking accuracy.
