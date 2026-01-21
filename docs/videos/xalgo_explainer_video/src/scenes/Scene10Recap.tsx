import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate, Sequence } from "remotion";
import { FadeInText } from "../components/AnimatedText";
import { BrainIcon, ShieldIcon, CheckIcon } from "../components/Icons";

const INNOVATIONS = [
  {
    title: "No Feature Engineering",
    description: "Transformer learns from engagement sequences",
    icon: BrainIcon,
    color: "#9B59B6",
  },
  {
    title: "Candidate Isolation",
    description: "Consistent, cacheable scores",
    icon: ShieldIcon,
    color: "#00BA7C",
  },
  {
    title: "18 Action Predictions",
    description: "Including negative signals for safety",
    icon: CheckIcon,
    color: "#1D9BF0",
  },
];

export const Scene10Recap: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  return (
    <AbsoluteFill className="bg-x-dark">
      {/* Title */}
      <div className="absolute top-16 left-0 right-0 text-center">
        <FadeInText
          text="Key Innovations"
          className="text-white text-5xl font-bold"
        />
      </div>

      <AbsoluteFill className="flex items-center justify-center">
        <div className="flex gap-8">
          {INNOVATIONS.map((innovation, i) => {
            const delay = fps * 0.3 + i * fps * 0.4;
            const entrance = spring({
              frame: frame - delay,
              fps,
              config: { damping: 15, stiffness: 200 },
            });

            const IconComponent = innovation.icon;

            return (
              <div
                key={innovation.title}
                className="w-80"
                style={{
                  opacity: entrance,
                  transform: `scale(${interpolate(entrance, [0, 1], [0.8, 1])}) translateY(${interpolate(entrance, [0, 1], [30, 0])}px)`,
                }}
              >
                {/* Card */}
                <div
                  className="h-72 rounded-2xl p-6 flex flex-col items-center"
                  style={{
                    backgroundColor: `${innovation.color}15`,
                    border: `2px solid ${innovation.color}40`,
                  }}
                >
                  {/* Icon */}
                  <div
                    className="w-20 h-20 rounded-full flex items-center justify-center mb-6"
                    style={{
                      backgroundColor: `${innovation.color}30`,
                    }}
                  >
                    <IconComponent size={40} color={innovation.color} />
                  </div>

                  {/* Title */}
                  <h3
                    className="text-xl font-bold text-center mb-4"
                    style={{ color: innovation.color }}
                  >
                    {innovation.title}
                  </h3>

                  {/* Description */}
                  <p className="text-x-gray text-center text-lg">
                    {innovation.description}
                  </p>
                </div>
              </div>
            );
          })}
        </div>
      </AbsoluteFill>

      {/* Summary */}
      <Sequence from={fps * 2} durationInFrames={fps * 6}>
        <div className="absolute bottom-16 left-0 right-0 text-center">
          <FadeInText
            text="Modern ML replaces manual feature engineering with learned patterns"
            className="text-x-gray text-2xl"
          />
        </div>
      </Sequence>
    </AbsoluteFill>
  );
};
