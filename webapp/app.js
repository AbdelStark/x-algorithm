const els = {
  tweetText: document.getElementById("tweetText"),
  followers: document.getElementById("followers"),
  following: document.getElementById("following"),
  accountAge: document.getElementById("accountAge"),
  engagementRate: document.getElementById("engagementRate"),
  postsPerDay: document.getElementById("postsPerDay"),
  hour: document.getElementById("hour"),
  media: document.getElementById("media"),
  hasLink: document.getElementById("hasLink"),
  verified: document.getElementById("verified"),
  useAi: document.getElementById("useAi"),
  novelty: document.getElementById("novelty"),
  timeliness: document.getElementById("timeliness"),
  audienceFit: document.getElementById("audienceFit"),
  topicSaturation: document.getElementById("topicSaturation"),
  controversy: document.getElementById("controversy"),
  sentiment: document.getElementById("sentiment"),
  noveltyValue: document.getElementById("noveltyValue"),
  timelinessValue: document.getElementById("timelinessValue"),
  audienceFitValue: document.getElementById("audienceFitValue"),
  topicSaturationValue: document.getElementById("topicSaturationValue"),
  controversyValue: document.getElementById("controversyValue"),
  sentimentValue: document.getElementById("sentimentValue"),
  simulate: document.getElementById("simulate"),
  scoreRing: document.getElementById("scoreRing"),
  scoreValue: document.getElementById("scoreValue"),
  scoreTier: document.getElementById("scoreTier"),
  weightedScore: document.getElementById("weightedScore"),
  impressionsTotal: document.getElementById("impressionsTotal"),
  impressionsIn: document.getElementById("impressionsIn"),
  impressionsOon: document.getElementById("impressionsOon"),
  uniqueEngagements: document.getElementById("uniqueEngagements"),
  uniqueRate: document.getElementById("uniqueRate"),
  actionVolume: document.getElementById("actionVolume"),
  actionRate: document.getElementById("actionRate"),
  likes: document.getElementById("likes"),
  replies: document.getElementById("replies"),
  reposts: document.getElementById("reposts"),
  shares: document.getElementById("shares"),
  signalList: document.getElementById("signalList"),
  suggestions: document.getElementById("suggestions"),
  probabilities: document.getElementById("probabilities"),
  warnings: document.getElementById("warnings"),
};

let pendingWarnings = [];

const sliderPairs = [
  [els.novelty, els.noveltyValue],
  [els.timeliness, els.timelinessValue],
  [els.audienceFit, els.audienceFitValue],
  [els.topicSaturation, els.topicSaturationValue],
  [els.controversy, els.controversyValue],
  [els.sentiment, els.sentimentValue],
];

sliderPairs.forEach(([slider, label]) => {
  const update = () => {
    label.textContent = Number(slider.value).toFixed(2);
  };
  slider.addEventListener("input", update);
  update();
});

els.simulate.addEventListener("click", async () => {
  const input = collectInput();
  if (els.useAi.checked) {
    const output = await simulateWithServer(input);
    if (output) {
      renderOutput(output);
      return;
    }
  }
  const output = simulate(input);
  renderOutput(output);
});

function collectInput() {
  return {
    text: els.tweetText.value || "",
    followers: parseNumber(els.followers.value, 1000),
    following: parseNumber(els.following.value, 500),
    accountAgeDays: parseNumber(els.accountAge.value, 365),
    avgEngagementRate: parseNumber(els.engagementRate.value, 0.02),
    postsPerDay: parseNumber(els.postsPerDay.value, 2.0),
    hourOfDay: clampInt(parseNumber(els.hour.value, 12), 0, 23),
    media: els.media.value,
    hasLink: els.hasLink.checked,
    verified: els.verified.checked,
    novelty: parseNumber(els.novelty.value, 0.5),
    timeliness: parseNumber(els.timeliness.value, 0.5),
    audienceFit: parseNumber(els.audienceFit.value, 0.6),
    topicSaturation: parseNumber(els.topicSaturation.value, 0.5),
    controversy: parseNumber(els.controversy.value, 0.3),
    sentiment: parseNumber(els.sentiment.value, 0.1),
  };
}

async function simulateWithServer(input) {
  try {
    const response = await fetch("/api/simulate", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        text: input.text,
        media: input.media,
        has_link: input.hasLink,
        followers: input.followers,
        following: input.following,
        account_age_days: input.accountAgeDays,
        avg_engagement_rate: input.avgEngagementRate,
        posts_per_day: input.postsPerDay,
        verified: input.verified,
        hour_of_day: input.hourOfDay,
        novelty: input.novelty,
        timeliness: input.timeliness,
        topic_saturation: input.topicSaturation,
        audience_fit: input.audienceFit,
        controversy: input.controversy,
        sentiment: input.sentiment,
        use_ai: true,
      }),
    });

    if (!response.ok) {
      pendingWarnings = [`AI server error: ${response.status}`];
      return null;
    }
    return await response.json();
  } catch (err) {
    pendingWarnings = [
      "AI server not reachable. Run `cargo run -- serve` and open http://localhost:8787.",
    ];
    return null;
  }
}

function extractTextFeatures(text) {
  let hashtags = 0;
  let mentions = 0;
  let questions = 0;
  let exclamations = 0;
  let emojiCount = 0;
  let uppercase = 0;
  let letters = 0;

  for (const ch of text) {
    if (ch === "#") hashtags += 1;
    if (ch === "@") mentions += 1;
    if (ch === "?") questions += 1;
    if (ch === "!") exclamations += 1;
    if (ch.codePointAt(0) > 127) emojiCount += 1;

    if (isAsciiLetter(ch)) {
      letters += 1;
      if (isUpper(ch)) uppercase += 1;
    }
  }

  const lowercase = text.toLowerCase();
  const urls = countMatches(lowercase, "http://")
    + countMatches(lowercase, "https://")
    + countMatches(lowercase, "www.");

  let wordTotal = 0;
  let wordCount = 0;
  for (const word of text.split(/\s+/)) {
    const len = Array.from(word).filter((c) => isAsciiLetter(c)).length;
    if (len > 0) {
      wordTotal += len;
      wordCount += 1;
    }
  }

  const avgWordLen = wordCount === 0 ? 0 : wordTotal / wordCount;
  const uppercaseRatio = letters === 0 ? 0 : uppercase / letters;
  const startsWithNumber = /^\s*\d/.test(text);

  const hookWords = [
    "how", "why", "what", "stop", "new", "breaking", "secret", "tips", "guide", "learn",
    "thread", "facts", "proof", "mistakes", "warning",
  ];
  const hasHookWord = hookWords.some((word) => lowercase.includes(word));

  const ctaShare = ["retweet", "repost", "share", "rt ", "boost"].some((word) => lowercase.includes(word));
  const ctaReply = ["thoughts", "what do you think", "agree", "disagree", "reply", "comment"].some(
    (word) => lowercase.includes(word)
  );

  return {
    charCount: Array.from(text).length,
    wordCount,
    hashtags,
    mentions,
    urls,
    questions,
    exclamations,
    emojiCount,
    uppercaseRatio,
    avgWordLen,
    startsWithNumber,
    hasHookWord,
    ctaShare,
    ctaReply,
  };
}

function simulate(input) {
  const features = extractTextFeatures(input.text);
  const mediaScore = mediaScoreFor(input.media);
  const hasLink = input.hasLink || features.urls > 0;
  const linkFlag = hasLink ? 1 : 0;

  const lengthScore = gaussian(features.charCount, 140, 70);
  const readabilityScore = gaussian(features.avgWordLen, 5, 2);

  const exclaimFactor = clamp01(features.exclamations / 3);
  const hashtagFactor = clamp01(features.hashtags / 5);
  const mentionFactor = clamp01(features.mentions / 4);

  const spamminess = clamp01(
    0.2 * exclaimFactor
      + 0.25 * hashtagFactor
      + 0.2 * mentionFactor
      + 0.3 * clamp01(features.uppercaseRatio / 0.4)
      + 0.2 * linkFlag
  );

  const hook = clamp01(
    0.35 * boolToNumber(features.questions > 0)
      + 0.2 * boolToNumber(features.exclamations > 0)
      + 0.25 * boolToNumber(features.startsWithNumber)
      + 0.2 * boolToNumber(features.hasHookWord)
  );

  const clarity = clamp01(0.5 * lengthScore + 0.3 * readabilityScore + 0.2 * (1 - spamminess));
  const novelty = clamp01(input.novelty);
  const timeliness = clamp01(input.timeliness);

  const shareability = clamp01(
    0.4 * hook
      + 0.3 * novelty
      + 0.2 * clarity
      + 0.1 * boolToNumber(features.ctaShare)
  );

  const contentQuality = clamp01(0.45 * clarity + 0.25 * hook + 0.2 * novelty + 0.1 * timeliness);

  const followersLog = log10Safe(input.followers + 1);
  const authorStrength = clamp01((followersLog - 2) / 3);

  const ratio = input.following === 0 ? 1 : input.followers / input.following;
  const ratioScore = clamp01(log10Safe(ratio + 1) / 2);

  const ageYears = input.accountAgeDays / 365;
  const ageScore = clamp01(ageYears / 5);

  const engScore = clamp01(input.avgEngagementRate / 0.06);
  const cadenceScore = gaussian(input.postsPerDay, 2, 2.5);
  const verifiedBonus = input.verified ? 0.1 : 0;

  const authorQuality = clamp01(
    0.35 * authorStrength + 0.2 * ageScore + 0.2 * engScore + 0.15 * ratioScore + 0.1 * cadenceScore + verifiedBonus
  );

  const topicSaturation = clamp01(input.topicSaturation);
  const audienceFit = clamp01(input.audienceFit);
  const audienceAlignment = clamp01(0.6 * audienceFit + 0.2 * (1 - topicSaturation) + 0.2 * ratioScore);

  const negativeSentiment = Math.max(0, -input.sentiment);
  const capsRisk = clamp01(features.uppercaseRatio / 0.35) * 0.2;
  const negativeRisk = clamp01(
    0.4 * clamp01(input.controversy) + 0.25 * spamminess + 0.15 * negativeSentiment + capsRisk + 0.1 * topicSaturation
  );

  const positiveSignal = clamp01(0.4 * contentQuality + 0.35 * authorQuality + 0.25 * audienceAlignment);
  const viralLift = clamp01(0.5 * hook + 0.3 * novelty + 0.2 * mediaScore);

  const base = -2.0 + 3.2 * positiveSignal + 1.4 * viralLift - 2.2 * negativeRisk;

  const hasQuestion = boolToNumber(features.questions > 0);
  const ctaReply = boolToNumber(features.ctaReply);
  const ctaShare = boolToNumber(features.ctaShare);
  const isVideo = input.media === "video" ? 1 : 0;
  const isImage = input.media === "image" || input.media === "gif" ? 1 : 0;

  const actions = {
    like: sigmoid(base + 0.6 * mediaScore + 0.2 * Math.max(0, input.sentiment)),
    reply: sigmoid(base - 0.2 * mediaScore + 0.6 * hasQuestion + 0.3 * input.controversy + 0.2 * ctaReply),
    repost: sigmoid(base + 0.6 * shareability + 0.3 * novelty - 0.3 * linkFlag + 0.1 * ctaShare),
    quote: sigmoid(base + 0.4 * input.controversy + 0.2 * novelty),
    click: sigmoid(base + 0.9 * linkFlag + 0.2 * hook),
    profileClick: sigmoid(base + 0.5 * authorQuality + 0.2 * novelty),
    videoView: sigmoid(base + 1.2 * isVideo + 0.2 * hook),
    photoExpand: sigmoid(base + 1.0 * isImage + 0.1 * hook),
    share: sigmoid(base + 0.5 * shareability + 0.2 * novelty),
    dwell: sigmoid(base + 0.2 * lengthScore + 0.4 * mediaScore - 0.2 * linkFlag),
    followAuthor: sigmoid(base + 0.6 * authorQuality + 0.2 * hook),
    notInterested: sigmoid(-1.0 + 2.2 * negativeRisk + 0.6 * topicSaturation - 0.8 * audienceAlignment),
    block: sigmoid(-2.0 + 2.6 * negativeRisk + 0.6 * input.controversy),
    mute: sigmoid(-1.8 + 2.3 * negativeRisk + 0.8 * topicSaturation),
    report: sigmoid(-2.4 + 2.8 * negativeRisk + 0.6 * input.controversy),
  };

  const weightedScore = weightedScoreFrom(actions);

  const timeScore = timeOfDayScore(input.hourOfDay);
  const activeFraction = 0.015 + 0.08 * timeScore;
  const impressionsIn = input.followers * activeFraction * Math.max(0, 0.6 + 0.4 * audienceAlignment);

  const oonSeed = 300 + 1400 * positiveSignal;
  const oonMultiplier = 1 + clamp01((weightedScore - 1) / 3) * 4;
  let impressionsOon =
    oonSeed * oonMultiplier * (0.5 + 0.5 * contentQuality) * (1 - 0.7 * topicSaturation) * (1 - 0.5 * negativeRisk);
  if (!Number.isFinite(impressionsOon) || impressionsOon < 0) impressionsOon = 0;

  const impressionsTotal = impressionsIn + impressionsOon;
  const actionVolumeRateValue = actionVolumeRate(actions);
  const uniqueEngagementRateValue = uniqueEngagementRate(actions);
  const expectedActionVolume = impressionsTotal * actionVolumeRateValue;
  const expectedUniqueEngagements = impressionsTotal * uniqueEngagementRateValue;

  const raw = (weightedScore - 1) * 0.8 + (log10Safe(impressionsTotal + 1) - 3) * 0.4;
  const score = 100 * sigmoid(raw);
  const tier = tierFromScore(score);

  const signals = {
    contentQuality,
    hook,
    authorQuality,
    audienceAlignment,
    negativeRisk,
    shareability,
    lengthScore,
    clarity,
    mediaScore,
    timeScore,
  };

  const suggestions = buildSuggestions(input, features, signals, actions, weightedScore);

  return {
    score,
    tier,
    weightedScore,
    impressionsIn,
    impressionsOon,
    impressionsTotal,
    expectedUniqueEngagements,
    expectedActionVolume,
    uniqueEngagementRate: uniqueEngagementRateValue,
    actionVolumeRate: actionVolumeRateValue,
    actions,
    signals,
    suggestions,
  };
}

function buildSuggestions(input, features, signals, actions, weightedScore) {
  const suggestions = [];

  if (features.charCount < 50) {
    suggestions.push("Add a clearer hook and more context; aim for ~80-200 characters.");
  }
  if (features.charCount > 260) {
    suggestions.push("Trim to under ~220 characters to improve early engagement velocity.");
  }
  if (features.hashtags > 3) {
    suggestions.push("Reduce hashtags to 1-2; too many can look spammy.");
  }
  if (features.mentions > 2) {
    suggestions.push("Limit mentions; too many can reduce reach and clarity.");
  }
  if (input.hasLink || features.urls > 0) {
    suggestions.push("Links often reduce in-feed engagement; consider moving the link to a reply.");
  }
  if (input.media === "none" && features.charCount < 160) {
    suggestions.push("Consider adding an image or video to boost dwell and shares.");
  }
  if (signals.hook < 0.35) {
    suggestions.push("Strengthen the first line with a question, surprising stat, or bold claim.");
  }
  if (signals.shareability < 0.4) {
    suggestions.push("Make it more shareable: concise takeaway, list, or strong opinion.");
  }
  if (signals.clarity < 0.5) {
    suggestions.push("Simplify wording; shorter words and fewer clauses improve scanability.");
  }
  if (features.uppercaseRatio > 0.3) {
    suggestions.push("Reduce ALL CAPS; it increases negative feedback signals.");
  }
  if (signals.negativeRisk > 0.55) {
    suggestions.push("Tone down contentious framing to reduce not-interested/report signals.");
  }
  if (input.topicSaturation > 0.6) {
    suggestions.push("High topic saturation; use a unique angle or niche framing.");
  }
  if (input.audienceFit < 0.5) {
    suggestions.push("Align the topic with follower interests to boost initial velocity.");
  }
  if (signals.timeScore < 0.4) {
    suggestions.push("Post during peak hours (around 9-11am or 7-9pm local).");
  }
  if (weightedScore < 0.8) {
    suggestions.push("Focus on increasing repost/share intent; that is a main driver of out-of-network reach.");
  }
  if (actions.reply < 0.02 && features.questions === 0) {
    suggestions.push("Invite replies with a direct question or a clear prompt.");
  }

  return suggestions;
}

function renderOutput(outputRaw) {
  const output = normalizeOutput(outputRaw);
  els.scoreValue.textContent = output.score.toFixed(1);
  els.scoreTier.textContent = output.tier;
  els.weightedScore.textContent = output.weightedScore.toFixed(2);
  els.impressionsTotal.textContent = formatNumber(output.impressionsTotal);
  els.impressionsIn.textContent = formatNumber(output.impressionsIn);
  els.impressionsOon.textContent = formatNumber(output.impressionsOon);
  els.uniqueEngagements.textContent = formatNumber(output.expectedUniqueEngagements);
  els.uniqueRate.textContent = formatPercent(output.uniqueEngagementRate);
  els.actionVolume.textContent = formatNumber(output.expectedActionVolume);
  els.actionRate.textContent = formatFloat(output.actionVolumeRate, 2);

  els.likes.textContent = formatNumber(output.impressionsTotal * output.actions.like);
  els.replies.textContent = formatNumber(output.impressionsTotal * output.actions.reply);
  els.reposts.textContent = formatNumber(output.impressionsTotal * output.actions.repost);
  els.shares.textContent = formatNumber(output.impressionsTotal * output.actions.share);

  const scorePercent = Math.min(100, Math.max(0, output.score));
  els.scoreRing.style.background = `conic-gradient(var(--accent) ${scorePercent}%, rgba(255, 255, 255, 0.4) 0%)`;

  const warningList = output.warnings && output.warnings.length ? output.warnings : pendingWarnings;
  renderWarnings(warningList);
  pendingWarnings = [];
  renderSignals(output.signals);
  renderSuggestions(output.suggestions);
  renderProbabilities(output.actions);
}

function renderSignals(signals) {
  const items = [
    ["Quality", signals.contentQuality],
    ["Hook", signals.hook],
    ["Author", signals.authorQuality],
    ["Audience", signals.audienceAlignment],
    ["Share", signals.shareability],
    ["Risk", signals.negativeRisk],
  ];

  els.signalList.innerHTML = "";
  items.forEach(([label, value]) => {
    const row = document.createElement("div");
    row.className = "signal";
    row.innerHTML = `
      <span>${label}</span>
      <div class="signal-bar"><span style="width:${(value * 100).toFixed(0)}%"></span></div>
      <strong>${value.toFixed(2)}</strong>
    `;
    els.signalList.appendChild(row);
  });
}

function renderWarnings(warnings) {
  if (!warnings || warnings.length === 0) {
    els.warnings.textContent = "";
    els.warnings.classList.remove("active");
    return;
  }
  els.warnings.textContent = warnings.join(" ");
  els.warnings.classList.add("active");
}

function normalizeOutput(raw) {
  const output = { ...raw };
  const actionsRaw = output.actions ?? {};
  const signalsRaw = output.signals ?? {};
  const actions = {
    like: actionsRaw.like ?? 0,
    reply: actionsRaw.reply ?? 0,
    repost: actionsRaw.repost ?? 0,
    quote: actionsRaw.quote ?? 0,
    click: actionsRaw.click ?? 0,
    profileClick: actionsRaw.profile_click ?? actionsRaw.profileClick ?? 0,
    videoView: actionsRaw.video_view ?? actionsRaw.videoView ?? 0,
    photoExpand: actionsRaw.photo_expand ?? actionsRaw.photoExpand ?? 0,
    share: actionsRaw.share ?? 0,
    dwell: actionsRaw.dwell ?? 0,
    followAuthor: actionsRaw.follow_author ?? actionsRaw.followAuthor ?? 0,
    notInterested: actionsRaw.not_interested ?? actionsRaw.notInterested ?? 0,
    block: actionsRaw.block ?? 0,
    mute: actionsRaw.mute ?? 0,
    report: actionsRaw.report ?? 0,
  };
  const signals = {
    contentQuality: signalsRaw.content_quality ?? signalsRaw.contentQuality ?? 0,
    hook: signalsRaw.hook ?? 0,
    authorQuality: signalsRaw.author_quality ?? signalsRaw.authorQuality ?? 0,
    negativeRisk: signalsRaw.negative_risk ?? signalsRaw.negativeRisk ?? 0,
    shareability: signalsRaw.shareability ?? 0,
    clarity: signalsRaw.clarity ?? 0,
    lengthScore: signalsRaw.length_score ?? signalsRaw.lengthScore ?? 0,
    audienceAlignment: signalsRaw.audience_alignment ?? signalsRaw.audienceAlignment ?? 0,
    mediaScore: signalsRaw.media_score ?? signalsRaw.mediaScore ?? 0,
    timeScore: signalsRaw.time_score ?? signalsRaw.timeScore ?? 0,
    novelty: signalsRaw.novelty ?? 0,
    timeliness: signalsRaw.timeliness ?? 0,
  };
  return {
    score: output.score ?? 0,
    tier: output.tier ?? "Low",
    weightedScore: output.weighted_score ?? output.weightedScore ?? 0,
    impressionsIn: output.impressions_in ?? output.impressionsIn ?? 0,
    impressionsOon: output.impressions_oon ?? output.impressionsOon ?? 0,
    impressionsTotal: output.impressions_total ?? output.impressionsTotal ?? 0,
    expectedUniqueEngagements:
      output.expected_unique_engagements ?? output.expectedUniqueEngagements ?? 0,
    expectedActionVolume:
      output.expected_action_volume ?? output.expectedActionVolume ?? 0,
    uniqueEngagementRate:
      output.unique_engagement_rate ?? output.uniqueEngagementRate ?? 0,
    actionVolumeRate: output.action_volume_rate ?? output.actionVolumeRate ?? 0,
    actions,
    signals,
    suggestions: output.suggestions ?? [],
    warnings: output.warnings ?? [],
  };
}

function renderSuggestions(suggestions) {
  els.suggestions.innerHTML = "";
  if (suggestions.length === 0) {
    const item = document.createElement("li");
    item.textContent = "No major blockers detected. Consider A/B testing different hooks.";
    els.suggestions.appendChild(item);
    return;
  }

  suggestions.forEach((suggestion) => {
    const item = document.createElement("li");
    item.textContent = suggestion;
    els.suggestions.appendChild(item);
  });
}

function renderProbabilities(actions) {
  const entries = [
    ["Like", actions.like],
    ["Reply", actions.reply],
    ["Repost", actions.repost],
    ["Quote", actions.quote],
    ["Share", actions.share],
    ["Click", actions.click],
    ["Profile click", actions.profileClick],
    ["Follow", actions.followAuthor],
    ["Video view", actions.videoView],
    ["Photo expand", actions.photoExpand],
    ["Dwell", actions.dwell],
    ["Not interested", actions.notInterested],
    ["Mute", actions.mute],
    ["Block", actions.block],
    ["Report", actions.report],
  ];

  els.probabilities.innerHTML = "";
  entries.forEach(([label, value]) => {
    const card = document.createElement("div");
    card.innerHTML = `<strong>${label}</strong><br />${formatPercent(value)}`;
    els.probabilities.appendChild(card);
  });
}

function mediaScoreFor(media) {
  switch (media) {
    case "image":
      return 0.4;
    case "gif":
      return 0.6;
    case "video":
      return 0.8;
    default:
      return 0.0;
  }
}

function timeOfDayScore(hour) {
  const morning = gaussian(hour, 9, 3.5);
  const evening = gaussian(hour, 20, 3.5);
  return Math.max(morning, evening);
}

function actionVolumeRate(actions) {
  return positiveActionProbs(actions).reduce((sum, value) => sum + value, 0);
}

function uniqueEngagementRate(actions) {
  let noneProbability = 1;
  positiveActionProbs(actions).forEach((value) => {
    noneProbability *= 1 - clamp01(value);
  });
  return clamp01(1 - noneProbability);
}

function positiveActionProbs(actions) {
  return [
    actions.like,
    actions.reply,
    actions.repost,
    actions.quote,
    actions.share,
    actions.click,
    actions.profileClick,
    actions.followAuthor,
    actions.videoView,
    actions.photoExpand,
  ];
}

function weightedScoreFrom(actions) {
  return (
    actions.like * 1.0
    + actions.reply * 1.6
    + actions.repost * 2.0
    + actions.quote * 1.7
    + actions.click * 0.4
    + actions.profileClick * 0.3
    + actions.videoView * 0.5
    + actions.photoExpand * 0.3
    + actions.share * 1.4
    + actions.dwell * 0.2
    + actions.followAuthor * 1.2
    + actions.notInterested * -2.5
    + actions.block * -5.0
    + actions.mute * -3.0
    + actions.report * -6.0
  );
}

function tierFromScore(score) {
  if (score < 35) return "Low";
  if (score < 55) return "Moderate";
  if (score < 75) return "High";
  if (score < 90) return "Very High";
  return "Breakout";
}

function gaussian(x, center, width) {
  if (width <= 0) return 0;
  const z = (x - center) / width;
  return Math.exp(-z * z);
}

function sigmoid(x) {
  return 1 / (1 + Math.exp(-x));
}

function clamp01(value) {
  if (!Number.isFinite(value)) return 0;
  return Math.min(1, Math.max(0, value));
}

function log10Safe(value) {
  if (value <= 0) return 0;
  return Math.log10(value);
}

function parseNumber(value, fallback) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : fallback;
}

function clampInt(value, min, max) {
  return Math.max(min, Math.min(max, Math.round(value)));
}

function boolToNumber(value) {
  return value ? 1 : 0;
}

function formatNumber(value) {
  const rounded = Math.max(0, Math.round(value));
  return rounded.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

function formatPercent(value) {
  return `${(value * 100).toFixed(1)}%`;
}

function formatFloat(value, digits) {
  return Number(value).toFixed(digits);
}

function countMatches(text, needle) {
  if (!needle) return 0;
  return text.split(needle).length - 1;
}

function isAsciiLetter(ch) {
  return /[A-Za-z]/.test(ch);
}

function isUpper(ch) {
  return /[A-Z]/.test(ch);
}

// Initialize with a sample output
const initial = simulate(collectInput());
renderOutput(initial);
