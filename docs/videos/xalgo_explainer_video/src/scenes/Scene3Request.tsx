import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate, Sequence } from "remotion";
import { FadeInText } from "../components/AnimatedText";
import { HeartIcon, RetweetIcon, ReplyIcon, UserIcon } from "../components/Icons";

export const Scene3Request: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // User avatar animation
  const userEntrance = spring({
    frame,
    fps,
    config: { damping: 200 },
  });

  // Data flows
  const flow1 = spring({ frame: frame - fps * 0.4, fps, config: { damping: 200 } });
  const flow2 = spring({ frame: frame - fps * 0.7, fps, config: { damping: 200 } });
  const flow3 = spring({ frame: frame - fps * 1.0, fps, config: { damping: 200 } });

  // Query assembly
  const queryAssembly = spring({
    frame: frame - fps * 1.5,
    fps,
    config: { damping: 15, stiffness: 200 },
  });

  return (
    <AbsoluteFill className="bg-x-dark">
      {/* Title */}
      <div className="absolute top-16 left-0 right-0 text-center">
        <FadeInText
          text="Step 1: Gathering Your Context"
          className="text-white text-4xl font-bold"
        />
      </div>

      <AbsoluteFill className="flex items-center justify-center">
        <div className="relative w-[800px] h-[500px]">
          {/* User avatar at center */}
          <div
            className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2"
            style={{
              opacity: userEntrance,
              transform: `translate(-50%, -50%) scale(${interpolate(userEntrance, [0, 1], [0.5, 1])})`,
            }}
          >
            <div className="w-32 h-32 rounded-full bg-gradient-to-br from-x-blue to-x-blue/50 flex items-center justify-center border-4 border-white/20">
              <UserIcon size={60} color="#FFFFFF" />
            </div>
            <div className="text-center mt-3">
              <span className="text-white text-lg font-medium">You</span>
            </div>
          </div>

          {/* Engagement History - Left */}
          <div
            className="absolute left-0 top-1/2 -translate-y-1/2"
            style={{
              opacity: flow1,
              transform: `translateY(-50%) translateX(${interpolate(flow1, [0, 1], [-30, 0])}px)`,
            }}
          >
            <div className="bg-x-pink/20 border border-x-pink rounded-xl p-6 w-48">
              <div className="text-x-pink font-semibold mb-3">Engagement History</div>
              <div className="flex gap-3 justify-center">
                <HeartIcon size={28} delay={fps * 0.5} />
                <RetweetIcon size={28} delay={fps * 0.6} />
                <ReplyIcon size={28} delay={fps * 0.7} />
              </div>
              <div className="text-x-gray text-sm mt-3 text-center">Last 50 actions</div>
            </div>
            {/* Connection line */}
            <svg
              className="absolute right-0 top-1/2 -translate-y-1/2 translate-x-full"
              width="80"
              height="4"
              style={{ opacity: flow1 }}
            >
              <line x1="0" y1="2" x2="80" y2="2" stroke="#F91880" strokeWidth="2" strokeDasharray="5,5">
                <animate attributeName="stroke-dashoffset" from="10" to="0" dur="0.5s" repeatCount="indefinite" />
              </line>
            </svg>
          </div>

          {/* Following List - Top */}
          <div
            className="absolute top-0 left-1/2 -translate-x-1/2"
            style={{
              opacity: flow2,
              transform: `translateX(-50%) translateY(${interpolate(flow2, [0, 1], [-30, 0])}px)`,
            }}
          >
            <div className="bg-x-blue/20 border border-x-blue rounded-xl p-6 w-48">
              <div className="text-x-blue font-semibold mb-3">Following List</div>
              <div className="flex -space-x-2 justify-center">
                {[0, 1, 2, 3, 4].map((i) => (
                  <div
                    key={i}
                    className="w-8 h-8 rounded-full bg-gradient-to-br from-x-blue/50 to-x-blue/30 border-2 border-x-dark"
                    style={{
                      opacity: interpolate(flow2, [0, 1], [0, 1]),
                    }}
                  />
                ))}
              </div>
              <div className="text-x-gray text-sm mt-3 text-center">~500 accounts</div>
            </div>
            {/* Connection line */}
            <svg
              className="absolute bottom-0 left-1/2 -translate-x-1/2 translate-y-full"
              width="4"
              height="60"
              style={{ opacity: flow2 }}
            >
              <line x1="2" y1="0" x2="2" y2="60" stroke="#1D9BF0" strokeWidth="2" strokeDasharray="5,5">
                <animate attributeName="stroke-dashoffset" from="10" to="0" dur="0.5s" repeatCount="indefinite" />
              </line>
            </svg>
          </div>

          {/* Preferences - Right */}
          <div
            className="absolute right-0 top-1/2 -translate-y-1/2"
            style={{
              opacity: flow3,
              transform: `translateY(-50%) translateX(${interpolate(flow3, [0, 1], [30, 0])}px)`,
            }}
          >
            <div className="bg-x-green/20 border border-x-green rounded-xl p-6 w-48">
              <div className="text-x-green font-semibold mb-3">Preferences</div>
              <div className="text-x-gray text-sm space-y-1">
                <div>Language: EN</div>
                <div>Topics: Tech, AI</div>
                <div>Muted: Politics</div>
              </div>
            </div>
            {/* Connection line */}
            <svg
              className="absolute left-0 top-1/2 -translate-y-1/2 -translate-x-full"
              width="80"
              height="4"
              style={{ opacity: flow3 }}
            >
              <line x1="80" y1="2" x2="0" y2="2" stroke="#00BA7C" strokeWidth="2" strokeDasharray="5,5">
                <animate attributeName="stroke-dashoffset" from="10" to="0" dur="0.5s" repeatCount="indefinite" />
              </line>
            </svg>
          </div>

          {/* Query Badge - Bottom */}
          <div
            className="absolute bottom-0 left-1/2 -translate-x-1/2"
            style={{
              opacity: queryAssembly,
              transform: `translateX(-50%) scale(${interpolate(queryAssembly, [0, 1], [0.5, 1])})`,
            }}
          >
            <div className="bg-x-gold/20 border-2 border-x-gold rounded-xl px-8 py-4">
              <div className="text-x-gold font-bold text-xl">QUERY</div>
              <div className="text-x-gray text-sm">"What would this user like?"</div>
            </div>
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
