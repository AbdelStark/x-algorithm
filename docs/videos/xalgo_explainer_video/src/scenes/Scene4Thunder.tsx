import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate, Sequence } from "remotion";
import { FadeInText } from "../components/AnimatedText";
import { LightningIcon, UserIcon } from "../components/Icons";
import { Counter } from "../components/Counter";

export const Scene4Thunder: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Thunder logo entrance
  const thunderEntrance = spring({
    frame,
    fps,
    config: { damping: 15, stiffness: 200 },
  });

  // Posts flowing
  const postsStart = fps * 0.8;
  const postFlowProgress = Math.max(0, frame - postsStart) / fps;

  return (
    <AbsoluteFill className="bg-x-dark">
      {/* Title */}
      <div className="absolute top-16 left-0 right-0 text-center">
        <FadeInText
          text="Source 1: Thunder - In-Network Posts"
          className="text-white text-4xl font-bold"
        />
      </div>

      <AbsoluteFill className="flex items-center justify-center">
        <div className="flex items-center gap-16">
          {/* Following grid */}
          <div
            className="flex flex-col items-center"
            style={{
              opacity: thunderEntrance,
              transform: `translateX(${interpolate(thunderEntrance, [0, 1], [-30, 0])}px)`,
            }}
          >
            <div className="text-x-gray text-lg mb-4">Accounts You Follow</div>
            <div className="grid grid-cols-5 gap-3">
              {Array.from({ length: 15 }, (_, i) => (
                <div
                  key={i}
                  className="w-12 h-12 rounded-full bg-gradient-to-br from-x-blue/50 to-x-blue/20 flex items-center justify-center border border-x-blue/30"
                  style={{
                    opacity: interpolate(
                      frame - fps * 0.3 - i * 2,
                      [0, 10],
                      [0, 1],
                      { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
                    ),
                    transform: `scale(${interpolate(
                      frame - fps * 0.3 - i * 2,
                      [0, 10],
                      [0.5, 1],
                      { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
                    )})`,
                  }}
                >
                  <UserIcon size={20} color="#1D9BF0" />
                </div>
              ))}
            </div>
            <div className="text-x-gray text-sm mt-3">~500 followed accounts</div>
          </div>

          {/* Thunder Engine */}
          <div className="flex flex-col items-center">
            {/* Posts flowing into Thunder */}
            <div className="relative h-40 w-32 mb-4 overflow-hidden">
              {Array.from({ length: 8 }, (_, i) => (
                <div
                  key={i}
                  className="absolute w-24 h-6 bg-x-blue/30 rounded"
                  style={{
                    left: 4,
                    top: ((frame * 2 + i * 20) % 160) - 20,
                    opacity: 0.5 + Math.sin((frame + i * 10) * 0.1) * 0.3,
                  }}
                />
              ))}
            </div>

            {/* Thunder box */}
            <div
              className="relative"
              style={{
                opacity: thunderEntrance,
                transform: `scale(${interpolate(thunderEntrance, [0, 1], [0.8, 1])})`,
              }}
            >
              <div className="w-48 h-48 bg-gradient-to-br from-x-gold/30 to-x-gold/10 border-2 border-x-gold rounded-2xl flex flex-col items-center justify-center">
                <LightningIcon size={60} color="#FFD93D" />
                <span className="text-x-gold text-2xl font-bold mt-2">THUNDER</span>
                <span className="text-x-gray text-sm mt-1">In-Memory Store</span>
              </div>
              {/* Speed indicator */}
              <div className="absolute -bottom-6 left-1/2 -translate-x-1/2 bg-x-gold/20 rounded-full px-4 py-1">
                <span className="text-x-gold text-sm font-mono">&lt;1ms</span>
              </div>
            </div>

            {/* Posts flowing out */}
            <div className="relative h-32 w-32 mt-8 overflow-hidden">
              {Array.from({ length: 6 }, (_, i) => (
                <div
                  key={i}
                  className="absolute w-20 h-5 bg-x-gold/40 rounded"
                  style={{
                    left: 6,
                    top: ((frame * 3 + i * 25) % 140) - 10,
                    opacity: 0.6 + Math.sin((frame + i * 15) * 0.1) * 0.2,
                  }}
                />
              ))}
            </div>
          </div>

          {/* Output */}
          <div
            className="flex flex-col items-center"
            style={{
              opacity: interpolate(frame - fps * 1.2, [0, fps * 0.5], [0, 1], {
                extrapolateLeft: "clamp",
                extrapolateRight: "clamp",
              }),
            }}
          >
            <div className="text-x-gray text-lg mb-4">In-Network Candidates</div>
            <div className="w-48 h-48 bg-x-blue/10 border border-x-blue/30 rounded-xl flex flex-col items-center justify-center">
              <Counter
                from={0}
                to={400}
                startFrame={fps * 1.2}
                durationFrames={fps * 0.8}
                prefix="~"
                className="text-x-gold text-5xl font-bold font-mono"
              />
              <span className="text-x-gray text-lg mt-2">posts</span>
            </div>
            <div className="text-x-blue text-sm mt-3">From your following list</div>
          </div>
        </div>
      </AbsoluteFill>

      {/* Description */}
      <Sequence from={fps * 2} durationInFrames={fps * 8}>
        <div className="absolute bottom-20 left-0 right-0 text-center">
          <FadeInText
            text="Lightning-fast retrieval of recent posts from accounts you follow"
            className="text-x-gray text-2xl"
          />
        </div>
      </Sequence>
    </AbsoluteFill>
  );
};
