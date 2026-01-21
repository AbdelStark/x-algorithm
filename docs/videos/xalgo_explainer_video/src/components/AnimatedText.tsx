import { useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";

type AnimatedTextProps = {
  text: string;
  delay?: number;
  className?: string;
  style?: React.CSSProperties;
};

export const FadeInText: React.FC<AnimatedTextProps> = ({
  text,
  delay = 0,
  className = "",
  style = {},
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const entrance = spring({
    frame: frame - delay,
    fps,
    config: { damping: 200 },
  });

  const opacity = interpolate(entrance, [0, 1], [0, 1]);
  const translateY = interpolate(entrance, [0, 1], [20, 0]);

  return (
    <div
      className={className}
      style={{
        opacity,
        transform: `translateY(${translateY}px)`,
        ...style,
      }}
    >
      {text}
    </div>
  );
};

export const TypewriterText: React.FC<AnimatedTextProps & { speed?: number }> = ({
  text,
  delay = 0,
  speed = 2, // characters per frame
  className = "",
  style = {},
}) => {
  const frame = useCurrentFrame();
  const adjustedFrame = Math.max(0, frame - delay);

  const visibleChars = Math.min(
    Math.floor(adjustedFrame / speed),
    text.length
  );

  const displayedText = text.slice(0, visibleChars);
  const showCursor = adjustedFrame > 0 && visibleChars < text.length;

  return (
    <span className={className} style={style}>
      {displayedText}
      {showCursor && (
        <span
          style={{
            opacity: Math.floor(frame * 0.1) % 2 === 0 ? 1 : 0,
          }}
        >
          |
        </span>
      )}
    </span>
  );
};

export const ScaleInText: React.FC<AnimatedTextProps> = ({
  text,
  delay = 0,
  className = "",
  style = {},
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const entrance = spring({
    frame: frame - delay,
    fps,
    config: { damping: 15, stiffness: 200 },
  });

  const scale = interpolate(entrance, [0, 1], [0, 1]);
  const opacity = interpolate(entrance, [0, 0.5], [0, 1], {
    extrapolateRight: "clamp",
  });

  return (
    <div
      className={className}
      style={{
        transform: `scale(${scale})`,
        opacity,
        ...style,
      }}
    >
      {text}
    </div>
  );
};
