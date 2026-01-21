import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate, Sequence } from "remotion";
import { FadeInText } from "../components/AnimatedText";

const WEIGHTS = [
  { action: "Like", weight: 1.0, color: "#F91880" },
  { action: "Reply", weight: 1.6, color: "#1D9BF0" },
  { action: "Repost", weight: 2.0, color: "#00BA7C" },
  { action: "Share", weight: 1.4, color: "#9B59B6" },
  { action: "Quote", weight: 1.7, color: "#FFD93D" },
  { action: "Click", weight: 0.4, color: "#8B949E" },
];

const NEGATIVE_WEIGHTS = [
  { action: "Not Interested", weight: -2.5, color: "#F91880" },
  { action: "Block", weight: -5.0, color: "#FF6B35" },
  { action: "Report", weight: -6.0, color: "#FF0000" },
];

export const Scene8WeightedScoring: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Animations
  const formulaEntrance = spring({ frame, fps, config: { damping: 200 } });
  const weightsEntrance = spring({ frame: frame - fps * 0.5, fps, config: { damping: 200 } });
  const negativeEntrance = spring({ frame: frame - fps * 1.2, fps, config: { damping: 200 } });
  const diversityEntrance = spring({ frame: frame - fps * 2, fps, config: { damping: 200 } });

  return (
    <AbsoluteFill className="bg-x-dark">
      {/* Title */}
      <div className="absolute top-12 left-0 right-0 text-center">
        <FadeInText
          text="Weighted Scoring"
          className="text-white text-4xl font-bold"
        />
        <FadeInText
          text="Combining predictions with business weights"
          className="text-x-gray text-xl mt-2"
          delay={fps * 0.3}
        />
      </div>

      <AbsoluteFill className="flex items-center justify-center pt-8">
        <div className="flex flex-col items-center gap-8">
          {/* Formula */}
          <div
            className="bg-white/5 border border-white/20 rounded-xl px-8 py-4"
            style={{
              opacity: formulaEntrance,
              transform: `scale(${interpolate(formulaEntrance, [0, 1], [0.9, 1])})`,
            }}
          >
            <span className="font-mono text-xl">
              <span className="text-white">Score = </span>
              <span className="text-x-gold">Σ</span>
              <span className="text-white"> (weight × P(action))</span>
            </span>
          </div>

          {/* Weights grid */}
          <div className="flex gap-16">
            {/* Positive weights */}
            <div
              className="flex flex-col"
              style={{
                opacity: weightsEntrance,
                transform: `translateX(${interpolate(weightsEntrance, [0, 1], [-20, 0])}px)`,
              }}
            >
              <div className="text-x-green font-semibold mb-4 text-center">
                Positive Signals
              </div>
              <div className="space-y-2">
                {WEIGHTS.map((w, i) => {
                  const delay = fps * 0.5 + i * 5;
                  const barWidth = interpolate(
                    frame - delay,
                    [0, fps * 0.3],
                    [0, (w.weight / 2) * 100],
                    { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
                  );

                  return (
                    <div key={w.action} className="flex items-center gap-3">
                      <div className="w-20 text-right">
                        <span className="text-white text-sm">{w.action}</span>
                      </div>
                      <div className="w-40 h-6 bg-white/10 rounded overflow-hidden">
                        <div
                          className="h-full rounded transition-all"
                          style={{
                            width: `${barWidth}%`,
                            backgroundColor: w.color,
                          }}
                        />
                      </div>
                      <span
                        className="text-lg font-bold font-mono w-12"
                        style={{ color: w.color }}
                      >
                        ×{w.weight}
                      </span>
                    </div>
                  );
                })}
              </div>
            </div>

            {/* Negative weights */}
            <div
              className="flex flex-col"
              style={{
                opacity: negativeEntrance,
                transform: `translateX(${interpolate(negativeEntrance, [0, 1], [20, 0])}px)`,
              }}
            >
              <div className="text-x-pink font-semibold mb-4 text-center">
                Negative Signals
              </div>
              <div className="space-y-2">
                {NEGATIVE_WEIGHTS.map((w, i) => {
                  const delay = fps * 1.2 + i * 5;
                  const barWidth = interpolate(
                    frame - delay,
                    [0, fps * 0.3],
                    [0, (Math.abs(w.weight) / 6) * 100],
                    { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
                  );

                  return (
                    <div key={w.action} className="flex items-center gap-3">
                      <div className="w-28 text-right">
                        <span className="text-white text-sm">{w.action}</span>
                      </div>
                      <div className="w-40 h-6 bg-white/10 rounded overflow-hidden">
                        <div
                          className="h-full rounded transition-all"
                          style={{
                            width: `${barWidth}%`,
                            backgroundColor: w.color,
                          }}
                        />
                      </div>
                      <span
                        className="text-lg font-bold font-mono w-12"
                        style={{ color: w.color }}
                      >
                        {w.weight}
                      </span>
                    </div>
                  );
                })}
              </div>

              {/* Warning */}
              <div className="mt-4 text-center text-x-pink text-sm">
                A likely block = -5 points!
              </div>
            </div>
          </div>

          {/* Author diversity */}
          <div
            className="flex items-center gap-6 bg-white/5 border border-white/20 rounded-xl px-6 py-4"
            style={{
              opacity: diversityEntrance,
              transform: `translateY(${interpolate(diversityEntrance, [0, 1], [20, 0])}px)`,
            }}
          >
            <div className="text-white">
              <span className="font-semibold">Author Diversity:</span>
              <span className="text-x-gray ml-2">score × 0.7^(times_seen)</span>
            </div>
            <div className="h-8 w-px bg-white/20" />
            <div className="flex items-center gap-2">
              {[1, 2, 3, 4].map((n) => (
                <div
                  key={n}
                  className="w-8 h-8 rounded bg-x-blue flex items-center justify-center text-white text-xs"
                  style={{ opacity: Math.pow(0.7, n - 1) }}
                >
                  #{n}
                </div>
              ))}
            </div>
            <span className="text-x-gray text-sm">Repeated authors get penalized</span>
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
