import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate, Sequence } from "remotion";
import { FadeInText } from "../components/AnimatedText";
import { BrainIcon, FireIcon } from "../components/Icons";
import { Counter } from "../components/Counter";

export const Scene5Phoenix: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Entrance animations
  const userTowerEntrance = spring({ frame: frame - fps * 0.3, fps, config: { damping: 200 } });
  const candidateTowerEntrance = spring({ frame: frame - fps * 0.6, fps, config: { damping: 200 } });
  const similarityEntrance = spring({ frame: frame - fps * 1.2, fps, config: { damping: 200 } });
  const resultEntrance = spring({ frame: frame - fps * 1.8, fps, config: { damping: 200 } });

  // Similarity animation
  const similarityPulse = Math.sin(frame * 0.15) * 0.5 + 0.5;

  return (
    <AbsoluteFill className="bg-x-dark">
      {/* Title */}
      <div className="absolute top-16 left-0 right-0 text-center">
        <FadeInText
          text="Source 2: Phoenix Retrieval - Out-of-Network"
          className="text-white text-4xl font-bold"
        />
      </div>

      <AbsoluteFill className="flex items-center justify-center">
        <div className="flex flex-col items-center gap-8">
          {/* Two towers */}
          <div className="flex items-start gap-32">
            {/* User Tower */}
            <div
              className="flex flex-col items-center"
              style={{
                opacity: userTowerEntrance,
                transform: `translateY(${interpolate(userTowerEntrance, [0, 1], [30, 0])}px)`,
              }}
            >
              <div className="text-x-blue font-semibold mb-4">User Tower</div>
              <div className="flex flex-col items-center gap-4">
                {/* Input */}
                <div className="bg-x-blue/20 border border-x-blue rounded-lg px-4 py-2">
                  <span className="text-white text-sm">You + History</span>
                </div>
                {/* Arrow down */}
                <div className="h-8 w-0.5 bg-x-blue" />
                {/* Transformer */}
                <div className="w-32 h-24 bg-gradient-to-b from-x-blue/40 to-x-blue/20 rounded-xl border border-x-blue flex flex-col items-center justify-center">
                  <BrainIcon size={32} color="#1D9BF0" />
                  <span className="text-x-gray text-xs mt-1">Transformer</span>
                </div>
                {/* Arrow down */}
                <div className="h-8 w-0.5 bg-x-blue" />
                {/* Embedding */}
                <div className="bg-x-blue/30 border-2 border-x-blue rounded-lg px-6 py-3">
                  <span className="text-x-blue font-mono text-sm">[768-dim vector]</span>
                </div>
              </div>
            </div>

            {/* Candidate Tower */}
            <div
              className="flex flex-col items-center"
              style={{
                opacity: candidateTowerEntrance,
                transform: `translateY(${interpolate(candidateTowerEntrance, [0, 1], [30, 0])}px)`,
              }}
            >
              <div className="text-x-gold font-semibold mb-4">Candidate Tower</div>
              <div className="flex flex-col items-center gap-4">
                {/* Input */}
                <div className="bg-x-gold/20 border border-x-gold rounded-lg px-4 py-2">
                  <span className="text-white text-sm">Millions of Posts</span>
                </div>
                {/* Arrow down */}
                <div className="h-8 w-0.5 bg-x-gold" />
                {/* MLP */}
                <div className="w-32 h-24 bg-gradient-to-b from-x-gold/40 to-x-gold/20 rounded-xl border border-x-gold flex flex-col items-center justify-center">
                  <FireIcon size={32} color="#FFD93D" />
                  <span className="text-x-gray text-xs mt-1">MLP Projection</span>
                </div>
                {/* Arrow down */}
                <div className="h-8 w-0.5 bg-x-gold" />
                {/* Embeddings */}
                <div className="bg-x-gold/30 border-2 border-x-gold rounded-lg px-6 py-3">
                  <span className="text-x-gold font-mono text-sm">[N × 768-dim]</span>
                </div>
              </div>
            </div>
          </div>

          {/* Similarity section */}
          <div
            className="flex flex-col items-center mt-4"
            style={{
              opacity: similarityEntrance,
              transform: `scale(${interpolate(similarityEntrance, [0, 1], [0.8, 1])})`,
            }}
          >
            {/* Connection lines */}
            <div className="flex items-center gap-8">
              <div className="w-32 h-0.5 bg-gradient-to-r from-x-blue to-transparent" />
              <div
                className="w-24 h-24 rounded-full flex items-center justify-center"
                style={{
                  background: `radial-gradient(circle, rgba(29, 155, 240, ${0.3 + similarityPulse * 0.3}) 0%, rgba(255, 217, 61, ${0.3 + similarityPulse * 0.3}) 100%)`,
                  boxShadow: `0 0 ${20 + similarityPulse * 20}px rgba(29, 155, 240, 0.5)`,
                }}
              >
                <span className="text-white font-bold">DOT</span>
              </div>
              <div className="w-32 h-0.5 bg-gradient-to-l from-x-gold to-transparent" />
            </div>
            <div className="text-x-gray text-sm mt-2">Cosine Similarity</div>
          </div>

          {/* Result */}
          <div
            className="flex items-center gap-8 mt-4"
            style={{
              opacity: resultEntrance,
              transform: `translateY(${interpolate(resultEntrance, [0, 1], [20, 0])}px)`,
            }}
          >
            <div className="text-x-gray">→ Top-K Similar Posts →</div>
            <div className="bg-x-green/20 border border-x-green rounded-xl px-6 py-4">
              <Counter
                from={0}
                to={400}
                startFrame={fps * 1.8}
                durationFrames={fps * 0.5}
                prefix="~"
                className="text-x-gold text-4xl font-bold font-mono"
              />
              <div className="text-x-gray text-sm mt-1">out-of-network posts</div>
            </div>
          </div>
        </div>
      </AbsoluteFill>

      {/* Description */}
      <Sequence from={fps * 2.5} durationInFrames={fps * 7}>
        <div className="absolute bottom-16 left-0 right-0 text-center">
          <FadeInText
            text="ML-powered discovery of posts from accounts you don't follow"
            className="text-x-gray text-2xl"
          />
        </div>
      </Sequence>
    </AbsoluteFill>
  );
};
