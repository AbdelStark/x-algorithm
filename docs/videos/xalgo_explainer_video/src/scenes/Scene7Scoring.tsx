import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate, Sequence } from "remotion";
import { FadeInText } from "../components/AnimatedText";
import { BrainIcon } from "../components/Icons";

const ACTIONS = [
  { name: "Like", color: "#F91880", positive: true },
  { name: "Reply", color: "#1D9BF0", positive: true },
  { name: "Repost", color: "#00BA7C", positive: true },
  { name: "Share", color: "#9B59B6", positive: true },
  { name: "Click", color: "#8B949E", positive: true },
  { name: "Dwell", color: "#8B949E", positive: true },
  { name: "Block", color: "#F91880", positive: false },
  { name: "Report", color: "#FF6B35", positive: false },
];

export const Scene7Scoring: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Section timings
  const modelEntrance = spring({ frame, fps, config: { damping: 200 } });
  const maskEntrance = spring({ frame: frame - fps * 0.8, fps, config: { damping: 200 } });
  const outputEntrance = spring({ frame: frame - fps * 1.8, fps, config: { damping: 200 } });

  return (
    <AbsoluteFill className="bg-x-dark">
      {/* Title */}
      <div className="absolute top-12 left-0 right-0 text-center">
        <FadeInText
          text="The Phoenix Transformer"
          className="text-white text-4xl font-bold"
        />
        <FadeInText
          text="Predicting 18 engagement actions"
          className="text-x-gray text-xl mt-2"
          delay={fps * 0.3}
        />
      </div>

      <AbsoluteFill className="flex items-center justify-center pt-8">
        <div className="flex gap-16">
          {/* Left: Input */}
          <div
            className="flex flex-col items-center"
            style={{
              opacity: modelEntrance,
              transform: `translateX(${interpolate(modelEntrance, [0, 1], [-20, 0])}px)`,
            }}
          >
            <div className="text-x-gray text-sm mb-4">INPUT</div>
            <div className="space-y-3">
              <div className="bg-x-blue/20 border border-x-blue rounded px-4 py-2 text-white text-sm">
                User Embedding
              </div>
              <div className="bg-x-gold/20 border border-x-gold rounded px-4 py-2 text-white text-sm">
                History (50 posts)
              </div>
              <div className="bg-x-green/20 border border-x-green rounded px-4 py-2 text-white text-sm">
                Candidates (~600)
              </div>
            </div>
          </div>

          {/* Center: Model with attention mask */}
          <div className="flex flex-col items-center">
            {/* Transformer */}
            <div
              className="w-48 h-32 bg-gradient-to-br from-x-blue/40 via-purple-500/30 to-x-gold/40 rounded-2xl border-2 border-white/20 flex flex-col items-center justify-center mb-6"
              style={{
                opacity: modelEntrance,
                transform: `scale(${interpolate(modelEntrance, [0, 1], [0.8, 1])})`,
              }}
            >
              <BrainIcon size={40} color="#FFFFFF" />
              <span className="text-white font-bold mt-2">PHOENIX</span>
              <span className="text-x-gray text-xs">Grok Transformer</span>
            </div>

            {/* Attention mask visualization */}
            <div
              className="relative"
              style={{
                opacity: maskEntrance,
                transform: `scale(${interpolate(maskEntrance, [0, 1], [0.8, 1])})`,
              }}
            >
              <div className="text-x-gray text-sm mb-2 text-center">Candidate Isolation</div>
              <div className="bg-x-dark border border-white/20 rounded-lg p-3">
                {/* 5x5 attention grid */}
                <div className="grid grid-cols-5 gap-1">
                  {/* Header row labels */}
                  <div className="text-x-gray text-[10px] text-center">-</div>
                  <div className="text-x-blue text-[10px] text-center">U</div>
                  <div className="text-x-gold text-[10px] text-center">H</div>
                  <div className="text-x-green text-[10px] text-center">C1</div>
                  <div className="text-x-green text-[10px] text-center">C2</div>

                  {/* Row: User */}
                  <div className="text-x-blue text-[10px] text-center">U</div>
                  {["#1D9BF0", "#FFD93D", "#333", "#333"].map((c, i) => (
                    <div
                      key={`u-${i}`}
                      className="w-6 h-6 rounded"
                      style={{
                        backgroundColor: c,
                        opacity: c === "#333" ? 0.3 : 0.8,
                      }}
                    />
                  ))}

                  {/* Row: History */}
                  <div className="text-x-gold text-[10px] text-center">H</div>
                  {["#1D9BF0", "#FFD93D", "#333", "#333"].map((c, i) => (
                    <div
                      key={`h-${i}`}
                      className="w-6 h-6 rounded"
                      style={{
                        backgroundColor: c,
                        opacity: c === "#333" ? 0.3 : 0.8,
                      }}
                    />
                  ))}

                  {/* Row: Candidate 1 */}
                  <div className="text-x-green text-[10px] text-center">C1</div>
                  {["#1D9BF0", "#FFD93D", "#00BA7C", "#333"].map((c, i) => (
                    <div
                      key={`c1-${i}`}
                      className="w-6 h-6 rounded"
                      style={{
                        backgroundColor: c,
                        opacity: c === "#333" ? 0.3 : 0.8,
                      }}
                    />
                  ))}

                  {/* Row: Candidate 2 */}
                  <div className="text-x-green text-[10px] text-center">C2</div>
                  {["#1D9BF0", "#FFD93D", "#333", "#00BA7C"].map((c, i) => (
                    <div
                      key={`c2-${i}`}
                      className="w-6 h-6 rounded"
                      style={{
                        backgroundColor: c,
                        opacity: c === "#333" ? 0.3 : 0.8,
                      }}
                    />
                  ))}
                </div>
              </div>
              <div className="text-x-gray text-[10px] text-center mt-2">
                Candidates can't see each other!
              </div>
            </div>
          </div>

          {/* Right: Output */}
          <div
            className="flex flex-col items-center"
            style={{
              opacity: outputEntrance,
              transform: `translateX(${interpolate(outputEntrance, [0, 1], [20, 0])}px)`,
            }}
          >
            <div className="text-x-gray text-sm mb-4">18 ACTION PREDICTIONS</div>
            <div className="grid grid-cols-2 gap-2">
              {ACTIONS.map((action, i) => {
                // Animated probability bar
                const prob = 0.3 + Math.sin((frame * 0.05 + i * 0.5)) * 0.2;
                const barDelay = fps * 1.8 + i * 3;
                const barEntrance = interpolate(
                  frame - barDelay,
                  [0, fps * 0.3],
                  [0, 1],
                  { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
                );

                return (
                  <div
                    key={action.name}
                    className="flex items-center gap-2"
                    style={{ opacity: barEntrance }}
                  >
                    <div className="w-16 text-right">
                      <span
                        className="text-xs font-medium"
                        style={{ color: action.positive ? action.color : "#F91880" }}
                      >
                        {action.name}
                      </span>
                    </div>
                    <div className="w-20 h-3 bg-white/10 rounded overflow-hidden">
                      <div
                        className="h-full rounded"
                        style={{
                          width: `${prob * 100}%`,
                          backgroundColor: action.positive ? action.color : "#F91880",
                        }}
                      />
                    </div>
                    <span className="text-x-gray text-[10px] w-8">
                      {(prob * 100).toFixed(0)}%
                    </span>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </AbsoluteFill>

      {/* Key insight */}
      <Sequence from={fps * 2.5} durationInFrames={fps * 12}>
        <div className="absolute bottom-12 left-0 right-0 text-center px-16">
          <FadeInText
            text="Each post is scored independently - scores are consistent and cacheable"
            className="text-x-blue text-xl"
          />
        </div>
      </Sequence>
    </AbsoluteFill>
  );
};
