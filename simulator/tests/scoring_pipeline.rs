use virality_sim::scoring::{ActionWeights, AuthorDiversityConfig, AuthorDiversityScorer, OonScorer, OonScorerConfig, ScoredCandidate, ScoringPipeline, WeightedScorer};
use virality_sim::ActionProbs;

fn empty_actions() -> ActionProbs {
    ActionProbs {
        like: 0.0,
        reply: 0.0,
        repost: 0.0,
        quote: 0.0,
        click: 0.0,
        profile_click: 0.0,
        video_view: 0.0,
        photo_expand: 0.0,
        share: 0.0,
        share_dm: 0.0,
        share_link: 0.0,
        dwell: 0.0,
        follow_author: 0.0,
        quoted_click: 0.0,
        not_interested: 0.0,
        block: 0.0,
        mute: 0.0,
        report: 0.0,
        dwell_time: 0.0,
    }
}

#[test]
fn weighted_scorer_uses_new_actions() {
    let mut actions = empty_actions();
    actions.share_dm = 1.0;
    actions.dwell_time = 2.0;

    let weights = ActionWeights::default();
    let scorer = WeightedScorer::new(weights, 6.0, 1.0);
    let score = scorer.score(&actions, None);

    let expected = 0.8 + 0.2;
    assert!((score - expected).abs() < 1e-6);
}

#[test]
fn weighted_scorer_applies_vqv_threshold() {
    let mut actions = empty_actions();
    actions.video_view = 1.0;

    let weights = ActionWeights::default();
    let scorer = WeightedScorer::new(weights, 6.0, 0.0);

    let short_score = scorer.score(&actions, Some(4.0));
    let long_score = scorer.score(&actions, Some(8.0));

    assert!((short_score - 0.0).abs() < 1e-6);
    assert!((long_score - 0.5).abs() < 1e-6);
}

#[test]
fn weighted_scorer_offsets_negative_scores() {
    let mut actions = empty_actions();
    actions.block = 1.0;

    let weights = ActionWeights::default();
    let scorer = WeightedScorer::new(weights, 6.0, 1.0);
    let score = scorer.score(&actions, None);

    assert!((score + 4.0).abs() < 1e-6);
}

#[test]
fn diversity_multiplier_decays_for_repeated_author() {
    let config = AuthorDiversityConfig { decay: 0.5, floor: 0.1 };
    let scorer = AuthorDiversityScorer::new(config);

    let mut candidates = vec![
        ScoredCandidate::new("post1".to_string(), "author".to_string(), false, None, empty_actions()),
        ScoredCandidate::new("post2".to_string(), "author".to_string(), false, None, empty_actions()),
    ];

    candidates[0].weighted_score = 2.0;
    candidates[1].weighted_score = 2.0;

    scorer.score(&mut candidates);

    assert!((candidates[0].diversity_multiplier - 1.0).abs() < 1e-6);
    assert!(candidates[1].diversity_multiplier < candidates[0].diversity_multiplier);
}

#[test]
fn oon_scorer_applies_multiplier() {
    let config = OonScorerConfig { multiplier: 0.5 };
    let scorer = OonScorer::new(config);

    let mut candidate = ScoredCandidate::new(
        "post".to_string(),
        "author".to_string(),
        true,
        None,
        empty_actions(),
    );
    candidate.score = 2.0;

    scorer.score(&mut candidate, true);

    assert!((candidate.score - 1.0).abs() < 1e-6);
    assert!((candidate.oon_multiplier - 0.5).abs() < 1e-6);
}

#[test]
fn scoring_pipeline_orders_by_final_score() {
    let scorer = ScoringPipeline::new(
        WeightedScorer::new(ActionWeights::default(), 6.0, 0.0),
        AuthorDiversityScorer::new(AuthorDiversityConfig::default()),
        OonScorer::new(OonScorerConfig::default()),
    );

    let mut high = empty_actions();
    high.like = 1.0;
    let mut low = empty_actions();
    low.block = 1.0;

    let mut candidates = vec![
        ScoredCandidate::new("low".to_string(), "a".to_string(), false, None, low),
        ScoredCandidate::new("high".to_string(), "b".to_string(), false, None, high),
    ];

    scorer.score(&mut candidates);

    assert_eq!(candidates[0].post_id, "high");
}

#[test]
fn scoring_pipeline_applies_diversity_and_oon() {
    let scorer = ScoringPipeline::new(
        WeightedScorer::new(ActionWeights::default(), 6.0, 0.0),
        AuthorDiversityScorer::new(AuthorDiversityConfig { decay: 0.5, floor: 0.1 }),
        OonScorer::new(OonScorerConfig { multiplier: 0.5 }),
    );

    let mut strong = empty_actions();
    strong.like = 1.0;
    let mut weak = empty_actions();
    weak.like = 0.8;

    let mut candidates = vec![
        ScoredCandidate::new("primary".to_string(), "author".to_string(), false, None, strong),
        ScoredCandidate::new("secondary".to_string(), "author".to_string(), true, None, weak),
    ];

    scorer.score(&mut candidates);

    let primary = &candidates[0];
    let secondary = &candidates[1];

    assert!((primary.diversity_multiplier - 1.0).abs() < 1e-6);
    assert!(secondary.diversity_multiplier < primary.diversity_multiplier);
    assert!(secondary.oon_multiplier < 1.0);
    assert!(primary.score > secondary.score);
    assert!((secondary.score - 0.22).abs() < 1e-6);
}
