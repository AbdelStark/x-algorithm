# X For You Algorithm - Explainer Video Script

**Duration**: ~4-5 minutes
**Style**: Modern, clean, educational with smooth animations
**Dimensions**: 1920x1080 (16:9)
**FPS**: 30

---

## Scene 1: Hook & Introduction (0:00 - 0:20)

### Visual
- Dark background with the X logo pulsing gently
- Numbers streaming past (representing millions of posts)
- Text animates in: "Every second, thousands of posts are created..."
- Camera zooms through the data stream

### Narration
"Every second, thousands of posts flood into X. But when you open your For You feed, you only see about 20-30 carefully selected posts. How does X decide what YOU see?"

### Animation Notes
- Particle effect for data stream
- Smooth fade transition to next scene

---

## Scene 2: The Big Picture (0:20 - 0:45)

### Visual
- Clean diagram showing the two-stage architecture
- Left side: Huge pool of "MILLIONS of posts"
- Arrow → "Retrieval" funnel → "~800 candidates"
- Arrow → "Ranking" funnel → "~30 posts"
- Arrow → Phone showing "Your Feed"

### Narration
"The algorithm uses a two-stage approach. First, RETRIEVAL quickly finds about 800 relevant candidates from millions of posts. Then, RANKING carefully scores and orders these candidates to pick the best 30 for your feed."

### Animation Notes
- Sequential reveal: Pool → Retrieval → Ranking → Feed
- Scale animation to show narrowing funnel
- Numbers animate counting down

---

## Scene 3: Your Request Begins (0:45 - 1:05)

### Visual
- User avatar at center
- Dotted lines connecting to:
  - "Engagement History" (heart, retweet, reply icons)
  - "Following List" (user icons)
  - "Preferences" (settings icon)
- All data flows into "Query" object

### Narration
"When you open X, the algorithm first gathers your context: who you follow, what you've engaged with recently, and your preferences. This becomes your 'query' - essentially asking 'what would THIS user like to see?'"

### Animation Notes
- Radial expansion from user avatar
- Icons pulse and flow toward center
- "Query" badge assembles from pieces

---

## Scene 4: Candidate Sourcing - Thunder (1:05 - 1:35)

### Visual
- Split screen showing TWO parallel paths
- LEFT: Lightning bolt icon "THUNDER"
  - Label: "In-Network Posts"
  - Shows posts from followed accounts flowing in
  - Counter: "~400 candidates"

### Narration
"Candidates come from two sources running in parallel. THUNDER is a high-speed in-memory store holding recent posts from everyone. It retrieves posts from accounts you follow - your 'in-network' content."

### Animation Notes
- Lightning effect for Thunder
- Grid of user avatars with posts flying off
- Speed lines showing performance

---

## Scene 5: Candidate Sourcing - Phoenix Retrieval (1:35 - 2:05)

### Visual
- RIGHT side of split: Phoenix bird icon
  - Label: "Out-of-Network Posts"
  - "Two-Tower Model" diagram:
    - User Tower: User → Transformer → Embedding
    - Candidate Tower: Posts → MLP → Embeddings
    - Dot product similarity visualization
  - Counter: "~400 candidates"
- Merge both sides: "Total: ~800 candidates"

### Narration
"PHOENIX uses machine learning to find posts you might like from accounts you DON'T follow. A 'two-tower' model creates an embedding of your interests, then finds similar posts from the entire platform through similarity search. Combined, you get about 800 candidate posts."

### Animation Notes
- Neural network animation for towers
- Vectors shooting toward each other
- Similarity glow effect when match

---

## Scene 6: The Filtering Gauntlet (2:05 - 2:35)

### Visual
- Pipeline visualization: 800 posts entering a series of gates
- Each gate has a label and shows some posts being removed:
  1. "Duplicates" (posts combine into one)
  2. "Too Old" (posts fade away)
  3. "Already Seen" (posts marked X)
  4. "Blocked Authors" (shield icon)
  5. "Muted Keywords" (speaker off icon)
- Counter decreasing: 800 → 750 → 680 → 620 → 590

### Narration
"Not all candidates make it through. Over TEN filters run sequentially: removing duplicates, posts older than 24 hours, content you've already seen, blocked or muted authors, and posts with keywords you've muted. Typically 100-200 posts get filtered out."

### Animation Notes
- Posts entering from left, some rejected at each gate
- Rejected posts fade out with X or crack effect
- Smooth counter animation

---

## Scene 7: The Magic - Phoenix Scoring (2:35 - 3:20)

### Visual
- Central "Phoenix" transformer model
- Input side: User embedding + History sequence + Candidate posts
- Attention mask visualization showing:
  - User/History can see each other (connected)
  - Candidates CANNOT see each other (isolated)
  - Candidates can only see User+History
- Output: 18 probability bars for each candidate

### Narration
"Here's where the magic happens. The Phoenix transformer takes your engagement history and each candidate, then predicts EIGHTEEN different actions: Will you like it? Reply? Repost? Share? Or... will you block or report it?"

"The KEY innovation is 'candidate isolation' - each post is scored independently, only seeing YOUR data, not other posts. This makes scores consistent and cacheable."

### Animation Notes
- Transformer attention visualization (animated dots)
- Isolation emphasized with barrier effect
- 18 action bars animate filling up
- Negative actions (block, report) in red

---

## Scene 8: Weighted Scoring (3:20 - 3:50)

### Visual
- Formula visualization:
  ```
  Score = 1.0×Like + 1.6×Reply + 2.0×Repost + ...
        - 2.5×NotInterested - 5.0×Block - 6.0×Report
  ```
- Weights shown as colored multipliers
- Positive weights in green, negative in red
- Bar chart showing how different actions contribute
- Plus: "Author Diversity" penalty (decay for repeat authors)
- Plus: "Out-of-Network" adjustment (×0.8)

### Narration
"These 18 probabilities are combined with weights. A repost is worth 2x a like. But negative signals subtract heavily - a likely block is minus 5 points! This weighted sum becomes the final score. The algorithm also penalizes repeat authors to keep your feed diverse."

### Animation Notes
- Weights pop in with emphasis
- Negative weights shake/pulse red
- Score bar filling up (or down)
- Diversity decay visualization

---

## Scene 9: Final Selection (3:50 - 4:10)

### Visual
- All candidates in a ranked list
- Top candidates glow gold
- Selection animation: top ~30 rise up
- Final validation checks flash by
- Posts arrange into a phone mockup showing the feed

### Narration
"Finally, the algorithm sorts all candidates by score and selects the top performers. One last round of safety checks, conversation deduplication, and... your personalized For You feed is ready."

### Animation Notes
- Sorting animation (posts rearranging)
- Top 30 lift off with glow
- Feed assembly in phone mockup
- Satisfying "complete" animation

---

## Scene 10: Key Innovations Recap (4:10 - 4:35)

### Visual
- Three cards animate in:
  1. "No Feature Engineering" - Transformer icon
     "The model learns patterns from sequences, not hand-crafted features"
  2. "Candidate Isolation" - Shield with checkmark
     "Consistent, cacheable scores"
  3. "18 Actions" - Grid of action icons
     "Including negative signals for safety"

### Narration
"Three key innovations make this work: The transformer learns relevance from engagement patterns, eliminating manual feature engineering. Candidate isolation ensures consistent scores. And predicting 18 actions - including negative ones - lets the algorithm balance engagement with safety."

### Animation Notes
- Cards flip in sequentially
- Icons animate within cards
- Subtle glow effect

---

## Scene 11: Outro (4:35 - 4:50)

### Visual
- X logo center
- Text: "The For You Algorithm"
- GitHub link: "Explore the code"
- Fade out

### Narration
"That's how X's For You algorithm works - a sophisticated pipeline balancing relevance, diversity, and safety to show you content you'll love."

### Animation Notes
- Elegant fade in of logo
- Text appears with subtle animation
- Clean fade to black

---

## Technical Specifications

### Color Palette
- Background: #0D1117 (dark)
- Primary: #1D9BF0 (X blue)
- Accent: #FFD93D (gold for highlights)
- Positive: #00BA7C (green)
- Negative: #F91880 (pink/red)
- Text: #FFFFFF
- Secondary text: #8B949E

### Typography
- Primary: Inter (clean, modern)
- Monospace: JetBrains Mono (for code/data)
- Weights: 400 (body), 600 (emphasis), 700 (headers)

### Animation Principles
- Ease-out for entrances
- Spring physics for interactive elements
- 0.3-0.5s transitions
- Stagger delays for sequential reveals (50-100ms)

### Components Needed
1. `XLogo` - Animated X logo
2. `DataStream` - Particle effect for data visualization
3. `Pipeline` - Funnel/flow diagram
4. `UserAvatar` - With engagement icons
5. `FilterGate` - For filtering visualization
6. `TransformerViz` - Attention mask visualization
7. `ScoreBar` - Animated probability bars
8. `ActionGrid` - 18 action icons
9. `WeightedFormula` - Math formula animation
10. `FeedMockup` - Phone with posts
11. `InfoCard` - For key points
12. `Counter` - Animated number counter

---

## Scene-by-Component Breakdown

| Scene | Duration | Key Components |
|-------|----------|----------------|
| 1. Hook | 20s | XLogo, DataStream |
| 2. Big Picture | 25s | Pipeline (2-stage) |
| 3. Your Request | 20s | UserAvatar, QueryBadge |
| 4. Thunder | 30s | LightningIcon, CandidateGrid |
| 5. Phoenix Retrieval | 30s | TwoTowerDiagram, SimilarityViz |
| 6. Filtering | 30s | FilterGate (x5), Counter |
| 7. Phoenix Scoring | 45s | TransformerViz, AttentionMask, ActionGrid |
| 8. Weighted Scoring | 30s | WeightedFormula, ScoreBar |
| 9. Final Selection | 20s | RankedList, FeedMockup |
| 10. Recap | 25s | InfoCard (x3) |
| 11. Outro | 15s | XLogo, GitHubLink |

**Total: ~290 seconds (~4:50)**
