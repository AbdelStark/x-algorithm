import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";
import { XLogo } from "../components/XLogo";
import { NumberStream } from "../components/DataStream";

export const Scene1Hook: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();

  // Logo entrance
  const logoEntrance = spring({
    frame,
    fps,
    config: { damping: 200 },
  });

  // Text animations - staggered entrance
  const text1Entrance = spring({
    frame: frame - fps * 0.5,
    fps,
    config: { damping: 200 },
  });

  const text2Entrance = spring({
    frame: frame - fps * 1.3,
    fps,
    config: { damping: 200 },
  });

  const text3Entrance = spring({
    frame: frame - fps * 2.5,
    fps,
    config: { damping: 200 },
  });

  // Zoom effect at end
  const zoomStart = durationInFrames - fps * 0.5;
  const zoom = interpolate(
    frame,
    [zoomStart, durationInFrames],
    [1, 1.5],
    { extrapolateLeft: "clamp" }
  );

  const fadeOut = interpolate(
    frame,
    [durationInFrames - fps * 0.3, durationInFrames],
    [1, 0],
    { extrapolateLeft: "clamp" }
  );

  return (
    <AbsoluteFill
      className="bg-x-dark"
      style={{
        transform: `scale(${zoom})`,
        opacity: fadeOut,
      }}
    >
      {/* Data stream background */}
      <NumberStream count={40} speed={3} />

      {/* Center content - all in one flex container */}
      <AbsoluteFill className="flex flex-col items-center justify-center">
        {/* X Logo */}
        <div
          style={{
            opacity: logoEntrance,
            transform: `scale(${interpolate(logoEntrance, [0, 1], [0.5, 1])})`,
          }}
        >
          <XLogo size={150} pulse />
        </div>

        {/* Text content - directly below logo, centered */}
        <div className="flex flex-col items-center mt-12 max-w-4xl text-center">
          {/* Text 1 */}
          <div
            style={{
              opacity: text1Entrance,
              transform: `translateY(${interpolate(text1Entrance, [0, 1], [20, 0])}px)`,
            }}
            className="text-white text-4xl font-semibold mb-6"
          >
            Every second, thousands of posts flood into X
          </div>

          {/* Text 2 */}
          <div
            style={{
              opacity: text2Entrance,
              transform: `translateY(${interpolate(text2Entrance, [0, 1], [20, 0])}px)`,
            }}
            className="text-x-gray text-2xl mb-8"
          >
            But you only see ~30 carefully selected posts
          </div>

          {/* Text 3 */}
          <div
            style={{
              opacity: text3Entrance,
              transform: `translateY(${interpolate(text3Entrance, [0, 1], [20, 0])}px)`,
            }}
            className="text-x-blue text-3xl font-bold"
          >
            How does X decide what YOU see?
          </div>
        </div>
      </AbsoluteFill>

      {/* Vignette effect */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          background:
            "radial-gradient(ellipse at center, transparent 40%, rgba(13, 17, 23, 0.8) 100%)",
          pointerEvents: "none",
        }}
      />
    </AbsoluteFill>
  );
};
