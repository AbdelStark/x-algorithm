import { useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";

type XLogoProps = {
  size?: number;
  color?: string;
  pulse?: boolean;
};

export const XLogo: React.FC<XLogoProps> = ({
  size = 120,
  color = "#FFFFFF",
  pulse = false,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Entrance animation
  const entrance = spring({
    frame,
    fps,
    config: { damping: 200 },
  });

  const scale = interpolate(entrance, [0, 1], [0.5, 1]);

  // Optional pulse effect
  const pulseScale = pulse
    ? 1 + Math.sin(frame * 0.1) * 0.02
    : 1;

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      style={{
        transform: `scale(${scale * pulseScale})`,
        opacity: entrance,
      }}
    >
      {/* X logo path */}
      <path
        d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"
        fill={color}
      />
    </svg>
  );
};
