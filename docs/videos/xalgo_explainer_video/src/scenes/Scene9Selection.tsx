import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate, Sequence, random } from "remotion";
import { FadeInText } from "../components/AnimatedText";
import { CheckIcon } from "../components/Icons";

export const Scene9Selection: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Generate posts with scores
  const posts = Array.from({ length: 20 }, (_, i) => ({
    id: i + 1,
    score: 0.95 - i * 0.03 + random(`score-${i}`) * 0.02,
    selected: i < 8,
  }));

  // Animation phases
  const sortStart = fps * 0.3;
  const selectStart = fps * 1;
  const phoneStart = fps * 1.5;

  // Sort animation progress
  const sortProgress = interpolate(
    frame - sortStart,
    [0, fps * 0.5],
    [0, 1],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
  );

  // Phone entrance
  const phoneEntrance = spring({
    frame: frame - phoneStart,
    fps,
    config: { damping: 200 },
  });

  return (
    <AbsoluteFill className="bg-x-dark">
      {/* Title */}
      <div className="absolute top-16 left-0 right-0 text-center">
        <FadeInText
          text="Final Selection: Top K"
          className="text-white text-4xl font-bold"
        />
      </div>

      <AbsoluteFill className="flex items-center justify-center">
        <div className="flex items-center gap-16">
          {/* Ranked list */}
          <div className="flex flex-col items-center">
            <div className="text-x-gray text-sm mb-4">Sorted by Score</div>
            <div className="grid grid-cols-4 gap-2">
              {posts.map((post, i) => {
                const delay = sortStart + i * 2;
                const entrance = interpolate(
                  frame - delay,
                  [0, fps * 0.2],
                  [0, 1],
                  { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
                );

                const isSelected = post.selected && frame > selectStart + i * 3;
                const selectedGlow = isSelected
                  ? interpolate(
                      frame - selectStart - i * 3,
                      [0, fps * 0.2],
                      [0, 1],
                      { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
                    )
                  : 0;

                return (
                  <div
                    key={post.id}
                    className="relative"
                    style={{
                      opacity: entrance,
                      transform: `scale(${interpolate(entrance, [0, 1], [0.8, 1])})`,
                    }}
                  >
                    <div
                      className="w-24 h-16 rounded-lg flex flex-col items-center justify-center"
                      style={{
                        backgroundColor: isSelected
                          ? `rgba(0, 186, 124, ${0.2 + selectedGlow * 0.2})`
                          : "rgba(255, 255, 255, 0.05)",
                        border: isSelected
                          ? `2px solid rgba(0, 186, 124, ${0.5 + selectedGlow * 0.5})`
                          : "1px solid rgba(255, 255, 255, 0.1)",
                        boxShadow: isSelected
                          ? `0 0 ${selectedGlow * 15}px rgba(0, 186, 124, 0.5)`
                          : "none",
                      }}
                    >
                      <span className="text-white text-xs font-mono">#{post.id}</span>
                      <span className="text-x-gold text-sm font-mono">
                        {post.score.toFixed(2)}
                      </span>
                    </div>
                    {isSelected && selectedGlow > 0.5 && (
                      <div className="absolute -top-1 -right-1">
                        <CheckIcon size={16} color="#00BA7C" />
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
            <div className="mt-4 flex items-center gap-4">
              <div className="flex items-center gap-2">
                <div className="w-4 h-4 bg-x-green/30 border border-x-green rounded" />
                <span className="text-x-gray text-sm">Selected (~30)</span>
              </div>
            </div>
          </div>

          {/* Arrow */}
          <div
            className="text-4xl text-x-gray"
            style={{
              opacity: phoneEntrance,
            }}
          >
            →
          </div>

          {/* Phone mockup */}
          <div
            style={{
              opacity: phoneEntrance,
              transform: `scale(${interpolate(phoneEntrance, [0, 1], [0.8, 1])})`,
            }}
          >
            <div className="w-48 h-80 bg-x-dark border-4 border-white/30 rounded-[2rem] flex flex-col items-center pt-4 overflow-hidden">
              {/* Notch */}
              <div className="w-20 h-2 bg-white/20 rounded-full mb-4" />

              {/* Feed items */}
              <div className="w-full px-3 space-y-2">
                {[0, 1, 2, 3, 4, 5].map((i) => {
                  const itemDelay = phoneStart + fps * 0.3 + i * 5;
                  const itemEntrance = interpolate(
                    frame - itemDelay,
                    [0, fps * 0.2],
                    [0, 1],
                    { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
                  );

                  return (
                    <div
                      key={i}
                      className="w-full h-10 bg-x-blue/30 rounded-lg"
                      style={{
                        opacity: itemEntrance,
                        transform: `translateY(${interpolate(itemEntrance, [0, 1], [10, 0])}px)`,
                      }}
                    />
                  );
                })}
              </div>
            </div>
            <div className="text-center mt-4">
              <span className="text-white font-medium">Your Feed</span>
            </div>
          </div>
        </div>
      </AbsoluteFill>

      {/* Description */}
      <Sequence from={fps * 2} durationInFrames={fps * 4}>
        <div className="absolute bottom-16 left-0 right-0 text-center">
          <FadeInText
            text="The top ~30 posts are delivered to your For You feed"
            className="text-x-gray text-2xl"
          />
        </div>
      </Sequence>
    </AbsoluteFill>
  );
};
