import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate, Sequence } from "remotion";
import { FadeInText } from "../components/AnimatedText";
import { Arrow } from "../components/Pipeline";
import { Counter } from "../components/Counter";

export const Scene2BigPicture: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Stage animations
  const stage1 = spring({ frame: frame - fps * 0.3, fps, config: { damping: 200 } });
  const stage2 = spring({ frame: frame - fps * 0.8, fps, config: { damping: 200 } });
  const stage3 = spring({ frame: frame - fps * 1.3, fps, config: { damping: 200 } });
  const stage4 = spring({ frame: frame - fps * 1.8, fps, config: { damping: 200 } });

  return (
    <AbsoluteFill className="bg-x-dark">
      {/* Title */}
      <div className="absolute top-16 left-0 right-0 text-center">
        <FadeInText
          text="The Two-Stage Architecture"
          className="text-white text-5xl font-bold"
        />
      </div>

      {/* Pipeline visualization */}
      <AbsoluteFill className="flex items-center justify-center">
        <div className="flex items-center gap-8">
          {/* Stage 1: Millions */}
          <div
            className="flex flex-col items-center"
            style={{
              opacity: stage1,
              transform: `translateY(${interpolate(stage1, [0, 1], [30, 0])}px)`,
            }}
          >
            <div className="w-48 h-48 rounded-full bg-gradient-to-br from-x-blue/30 to-x-blue/10 border-2 border-x-blue flex flex-col items-center justify-center">
              <span className="text-x-gold text-4xl font-bold font-mono">MILLIONS</span>
              <span className="text-x-gray text-lg mt-2">of posts</span>
            </div>
          </div>

          {/* Arrow 1 */}
          <div style={{ opacity: stage2 }}>
            <Arrow direction="right" length={60} />
          </div>

          {/* Stage 2: Retrieval */}
          <div
            className="flex flex-col items-center"
            style={{
              opacity: stage2,
              transform: `translateY(${interpolate(stage2, [0, 1], [30, 0])}px)`,
            }}
          >
            <div
              className="w-40 h-32 flex flex-col items-center justify-center rounded-xl"
              style={{
                background: "linear-gradient(180deg, rgba(29, 155, 240, 0.3) 0%, rgba(29, 155, 240, 0.1) 100%)",
                borderTop: "3px solid #1D9BF0",
                borderLeft: "2px solid #1D9BF0",
                borderRight: "2px solid #1D9BF0",
                borderBottom: "none",
                clipPath: "polygon(10% 0%, 90% 0%, 100% 100%, 0% 100%)",
              }}
            >
              <span className="text-white text-xl font-bold">RETRIEVAL</span>
              <span className="text-x-gray text-sm">Fast & Broad</span>
            </div>
            <div className="mt-4 text-center">
              <Counter
                from={0}
                to={800}
                startFrame={fps * 0.8}
                durationFrames={fps * 0.5}
                prefix="~"
                className="text-x-gold text-3xl font-bold font-mono"
              />
              <div className="text-x-gray text-sm">candidates</div>
            </div>
          </div>

          {/* Arrow 2 */}
          <div style={{ opacity: stage3 }}>
            <Arrow direction="right" length={60} />
          </div>

          {/* Stage 3: Ranking */}
          <div
            className="flex flex-col items-center"
            style={{
              opacity: stage3,
              transform: `translateY(${interpolate(stage3, [0, 1], [30, 0])}px)`,
            }}
          >
            <div
              className="w-32 h-28 flex flex-col items-center justify-center rounded-xl"
              style={{
                background: "linear-gradient(180deg, rgba(255, 217, 61, 0.3) 0%, rgba(255, 217, 61, 0.1) 100%)",
                borderTop: "3px solid #FFD93D",
                borderLeft: "2px solid #FFD93D",
                borderRight: "2px solid #FFD93D",
                borderBottom: "none",
                clipPath: "polygon(15% 0%, 85% 0%, 100% 100%, 0% 100%)",
              }}
            >
              <span className="text-white text-xl font-bold">RANKING</span>
              <span className="text-x-gray text-sm">Slow & Precise</span>
            </div>
            <div className="mt-4 text-center">
              <Counter
                from={0}
                to={30}
                startFrame={fps * 1.3}
                durationFrames={fps * 0.5}
                prefix="~"
                className="text-x-gold text-3xl font-bold font-mono"
              />
              <div className="text-x-gray text-sm">posts</div>
            </div>
          </div>

          {/* Arrow 3 */}
          <div style={{ opacity: stage4 }}>
            <Arrow direction="right" length={60} />
          </div>

          {/* Stage 4: Feed */}
          <div
            className="flex flex-col items-center"
            style={{
              opacity: stage4,
              transform: `translateY(${interpolate(stage4, [0, 1], [30, 0])}px)`,
            }}
          >
            {/* Phone mockup */}
            <div className="w-32 h-56 bg-x-dark border-4 border-white/20 rounded-3xl flex flex-col items-center justify-start pt-4 overflow-hidden">
              <div className="w-16 h-1 bg-white/30 rounded-full mb-4" />
              {/* Feed items */}
              {[0, 1, 2, 3].map((i) => (
                <div
                  key={i}
                  className="w-24 h-8 bg-x-blue/30 rounded mb-2"
                  style={{
                    opacity: interpolate(
                      frame - fps * 2 - i * 5,
                      [0, 10],
                      [0, 1],
                      { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
                    ),
                  }}
                />
              ))}
            </div>
            <span className="text-white text-lg font-medium mt-4">Your Feed</span>
          </div>
        </div>
      </AbsoluteFill>

      {/* Bottom text */}
      <Sequence from={fps * 2} durationInFrames={fps * 6}>
        <div className="absolute bottom-20 left-0 right-0 text-center">
          <FadeInText
            text="Find relevant candidates quickly, then rank them precisely"
            className="text-x-gray text-2xl"
          />
        </div>
      </Sequence>
    </AbsoluteFill>
  );
};
