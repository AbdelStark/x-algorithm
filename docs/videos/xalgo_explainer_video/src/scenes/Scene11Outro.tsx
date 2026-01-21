import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { XLogo } from "../components/XLogo";

export const Scene11Outro: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();

  // Entrance animations
  const logoEntrance = spring({
    frame,
    fps,
    config: { damping: 200 },
  });

  const textEntrance = spring({
    frame: frame - fps * 0.3,
    fps,
    config: { damping: 200 },
  });

  const linkEntrance = spring({
    frame: frame - fps * 0.6,
    fps,
    config: { damping: 200 },
  });

  // Fade out at end
  const fadeOut = interpolate(
    frame,
    [durationInFrames - fps * 0.3, durationInFrames],
    [1, 0],
    { extrapolateLeft: "clamp" }
  );

  return (
    <AbsoluteFill className="bg-x-dark" style={{ opacity: fadeOut }}>
      {/* Gradient background */}
      <div
        className="absolute inset-0"
        style={{
          background:
            "radial-gradient(ellipse at center, rgba(29, 155, 240, 0.1) 0%, transparent 70%)",
        }}
      />

      <AbsoluteFill className="flex flex-col items-center justify-center">
        {/* X Logo */}
        <div
          style={{
            opacity: logoEntrance,
            transform: `scale(${interpolate(logoEntrance, [0, 1], [0.5, 1])})`,
          }}
        >
          <XLogo size={120} pulse />
        </div>

        {/* Title */}
        <div
          className="mt-8"
          style={{
            opacity: textEntrance,
            transform: `translateY(${interpolate(textEntrance, [0, 1], [20, 0])}px)`,
          }}
        >
          <h1 className="text-white text-4xl font-bold text-center">
            The For You Algorithm
          </h1>
          <p className="text-x-gray text-xl text-center mt-4">
            Relevance • Diversity • Safety
          </p>
        </div>

        {/* GitHub link */}
        <div
          className="mt-12 bg-white/10 border border-white/20 rounded-xl px-8 py-4"
          style={{
            opacity: linkEntrance,
            transform: `translateY(${interpolate(linkEntrance, [0, 1], [20, 0])}px)`,
          }}
        >
          <span className="text-x-gray text-lg">Explore the code:</span>
          <span className="text-x-blue text-lg ml-2 font-mono">
            github.com/xai-org/x-algorithm
          </span>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
