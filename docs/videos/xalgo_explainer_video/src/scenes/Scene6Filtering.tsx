import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate, Sequence } from "remotion";
import { FadeInText } from "../components/AnimatedText";
import { FilterIcon, XMarkIcon } from "../components/Icons";
import { Counter } from "../components/Counter";

const FILTERS = [
  { name: "Duplicates", removed: 10 },
  { name: "Too Old (>24h)", removed: 50 },
  { name: "Already Seen", removed: 100 },
  { name: "Blocked Authors", removed: 15 },
  { name: "Muted Keywords", removed: 25 },
];

export const Scene6Filtering: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Calculate running totals
  let runningCount = 800;
  const filterCounts = FILTERS.map((f) => {
    const before = runningCount;
    runningCount -= f.removed;
    return { ...f, before, after: runningCount };
  });

  return (
    <AbsoluteFill className="bg-x-dark">
      {/* Title */}
      <div className="absolute top-16 left-0 right-0 text-center">
        <FadeInText
          text="The Filtering Gauntlet"
          className="text-white text-4xl font-bold"
        />
        <FadeInText
          text="10+ filters remove ineligible posts"
          className="text-x-gray text-xl mt-2"
          delay={fps * 0.3}
        />
      </div>

      <AbsoluteFill className="flex items-center justify-center pt-16">
        <div className="flex items-center gap-4">
          {/* Starting count */}
          <div className="flex flex-col items-center mr-8">
            <Counter
              from={0}
              to={800}
              startFrame={0}
              durationFrames={fps * 0.3}
              prefix="~"
              className="text-x-gold text-4xl font-bold font-mono"
            />
            <span className="text-x-gray text-sm">candidates</span>
          </div>

          {/* Filter gates */}
          {filterCounts.map((filter, i) => {
            const filterStart = fps * 0.5 + i * fps * 0.4;
            const entrance = spring({
              frame: frame - filterStart,
              fps,
              config: { damping: 200 },
            });

            const rejectAnimation = interpolate(
              frame - filterStart - fps * 0.2,
              [0, fps * 0.2],
              [0, 1],
              { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
            );

            return (
              <div key={i} className="flex items-center gap-4">
                {/* Arrow */}
                <div
                  style={{ opacity: entrance }}
                  className="w-8 h-0.5 bg-x-gray"
                />

                {/* Filter gate */}
                <div
                  className="relative"
                  style={{
                    opacity: entrance,
                    transform: `scale(${interpolate(entrance, [0, 1], [0.8, 1])})`,
                  }}
                >
                  {/* Gate */}
                  <div className="w-32 h-40 bg-gradient-to-b from-x-pink/30 to-transparent border-t-4 border-x-pink rounded-t-xl flex flex-col items-center pt-4">
                    <FilterIcon size={24} color="#F91880" />
                    <div className="text-white text-xs font-medium text-center mt-2 px-2">
                      {filter.name}
                    </div>
                  </div>

                  {/* Rejected posts animation */}
                  <div
                    className="absolute -right-4 top-1/2 -translate-y-1/2"
                    style={{
                      opacity: rejectAnimation,
                      transform: `translateX(${rejectAnimation * 20}px) rotate(${rejectAnimation * 15}deg)`,
                    }}
                  >
                    <div className="flex items-center gap-1">
                      <XMarkIcon size={16} color="#F91880" />
                      <span className="text-x-pink text-xs font-mono">
                        -{filter.removed}
                      </span>
                    </div>
                  </div>

                  {/* Running count below */}
                  <div className="absolute -bottom-8 left-1/2 -translate-x-1/2">
                    <Counter
                      from={filter.before}
                      to={filter.after}
                      startFrame={filterStart + fps * 0.2}
                      durationFrames={fps * 0.15}
                      className="text-x-gold text-lg font-mono"
                    />
                  </div>
                </div>
              </div>
            );
          })}

          {/* Final arrow */}
          <div
            className="w-8 h-0.5 bg-x-gray"
            style={{
              opacity: interpolate(
                frame - fps * 2.5,
                [0, fps * 0.2],
                [0, 1],
                { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
              ),
            }}
          />

          {/* Final count */}
          <div
            className="flex flex-col items-center ml-8"
            style={{
              opacity: interpolate(
                frame - fps * 2.5,
                [0, fps * 0.3],
                [0, 1],
                { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
              ),
            }}
          >
            <div className="bg-x-green/20 border-2 border-x-green rounded-xl px-6 py-4">
              <Counter
                from={800}
                to={600}
                startFrame={fps * 2.5}
                durationFrames={fps * 0.3}
                prefix="~"
                className="text-x-green text-4xl font-bold font-mono"
              />
              <div className="text-x-gray text-sm mt-1">remaining</div>
            </div>
          </div>
        </div>
      </AbsoluteFill>

      {/* Description */}
      <Sequence from={fps * 3} durationInFrames={fps * 6}>
        <div className="absolute bottom-16 left-0 right-0 text-center">
          <FadeInText
            text="Quality control ensures you only see relevant, fresh content"
            className="text-x-gray text-2xl"
          />
        </div>
      </Sequence>
    </AbsoluteFill>
  );
};
