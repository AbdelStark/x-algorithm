import { useCurrentFrame, useVideoConfig, interpolate } from "remotion";

type CounterProps = {
  from: number;
  to: number;
  startFrame?: number;
  durationFrames?: number;
  prefix?: string;
  suffix?: string;
  className?: string;
  style?: React.CSSProperties;
  format?: (n: number) => string;
};

export const Counter: React.FC<CounterProps> = ({
  from,
  to,
  startFrame = 0,
  durationFrames,
  prefix = "",
  suffix = "",
  className = "",
  style = {},
  format = (n) => Math.round(n).toLocaleString(),
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const duration = durationFrames ?? fps; // Default 1 second

  const value = interpolate(
    frame,
    [startFrame, startFrame + duration],
    [from, to],
    {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    }
  );

  return (
    <span className={className} style={style}>
      {prefix}
      {format(value)}
      {suffix}
    </span>
  );
};

// Specialized counter for candidate count
export const CandidateCounter: React.FC<{
  from: number;
  to: number;
  startFrame?: number;
  durationFrames?: number;
  label?: string;
}> = ({ from, to, startFrame = 0, durationFrames, label }) => {
  return (
    <div className="flex flex-col items-center">
      <Counter
        from={from}
        to={to}
        startFrame={startFrame}
        durationFrames={durationFrames}
        prefix="~"
        className="text-5xl font-bold text-x-gold font-mono"
      />
      {label && <span className="text-xl text-x-gray mt-2">{label}</span>}
    </div>
  );
};
