import "./index.css";
import { Composition, Folder } from "remotion";
import { XAlgoExplainer } from "./XAlgoExplainer";
import { Scene1Hook } from "./scenes/Scene1Hook";
import { Scene2BigPicture } from "./scenes/Scene2BigPicture";
import { Scene3Request } from "./scenes/Scene3Request";
import { Scene4Thunder } from "./scenes/Scene4Thunder";
import { Scene5Phoenix } from "./scenes/Scene5Phoenix";
import { Scene6Filtering } from "./scenes/Scene6Filtering";
import { Scene7Scoring } from "./scenes/Scene7Scoring";
import { Scene8WeightedScoring } from "./scenes/Scene8WeightedScoring";
import { Scene9Selection } from "./scenes/Scene9Selection";
import { Scene10Recap } from "./scenes/Scene10Recap";
import { Scene11Outro } from "./scenes/Scene11Outro";

// Video constants
const FPS = 30;
const WIDTH = 1920;
const HEIGHT = 1080;

// Scene durations in seconds (tight - end ~1-2s after animation completes)
const SCENE_DURATIONS = {
  hook: 7,        // Logo + 3 text lines, ~5s animation + 2s buffer
  bigPicture: 8,  // Pipeline stages animate in ~6s
  request: 6,     // User context gathers ~4s
  thunder: 6,     // Thunder animation ~4s
  phoenix: 7,     // Two towers + similarity ~5s
  filtering: 7,   // Filters cascade ~5s
  scoring: 8,     // Transformer + outputs ~6s
  weightedScoring: 7, // Weights + bars ~5s
  selection: 6,   // Sort + phone ~4s
  recap: 6,       // 3 cards ~4s
  outro: 4,       // Logo + text ~2s
};

// Convert to frames
const SCENE_FRAMES = Object.fromEntries(
  Object.entries(SCENE_DURATIONS).map(([key, seconds]) => [key, seconds * FPS])
) as Record<keyof typeof SCENE_DURATIONS, number>;

// Total duration
const TOTAL_FRAMES = Object.values(SCENE_FRAMES).reduce((a, b) => a + b, 0);

export const RemotionRoot: React.FC = () => {
  return (
    <>
      {/* Main composition */}
      <Composition
        id="XAlgoExplainer"
        component={XAlgoExplainer}
        durationInFrames={TOTAL_FRAMES}
        fps={FPS}
        width={WIDTH}
        height={HEIGHT}
        defaultProps={{
          sceneFrames: SCENE_FRAMES,
        }}
      />

      {/* Individual scenes for preview */}
      <Folder name="Scenes">
        <Composition
          id="Scene1-Hook"
          component={Scene1Hook}
          durationInFrames={SCENE_FRAMES.hook}
          fps={FPS}
          width={WIDTH}
          height={HEIGHT}
        />
        <Composition
          id="Scene2-BigPicture"
          component={Scene2BigPicture}
          durationInFrames={SCENE_FRAMES.bigPicture}
          fps={FPS}
          width={WIDTH}
          height={HEIGHT}
        />
        <Composition
          id="Scene3-Request"
          component={Scene3Request}
          durationInFrames={SCENE_FRAMES.request}
          fps={FPS}
          width={WIDTH}
          height={HEIGHT}
        />
        <Composition
          id="Scene4-Thunder"
          component={Scene4Thunder}
          durationInFrames={SCENE_FRAMES.thunder}
          fps={FPS}
          width={WIDTH}
          height={HEIGHT}
        />
        <Composition
          id="Scene5-Phoenix"
          component={Scene5Phoenix}
          durationInFrames={SCENE_FRAMES.phoenix}
          fps={FPS}
          width={WIDTH}
          height={HEIGHT}
        />
        <Composition
          id="Scene6-Filtering"
          component={Scene6Filtering}
          durationInFrames={SCENE_FRAMES.filtering}
          fps={FPS}
          width={WIDTH}
          height={HEIGHT}
        />
        <Composition
          id="Scene7-Scoring"
          component={Scene7Scoring}
          durationInFrames={SCENE_FRAMES.scoring}
          fps={FPS}
          width={WIDTH}
          height={HEIGHT}
        />
        <Composition
          id="Scene8-WeightedScoring"
          component={Scene8WeightedScoring}
          durationInFrames={SCENE_FRAMES.weightedScoring}
          fps={FPS}
          width={WIDTH}
          height={HEIGHT}
        />
        <Composition
          id="Scene9-Selection"
          component={Scene9Selection}
          durationInFrames={SCENE_FRAMES.selection}
          fps={FPS}
          width={WIDTH}
          height={HEIGHT}
        />
        <Composition
          id="Scene10-Recap"
          component={Scene10Recap}
          durationInFrames={SCENE_FRAMES.recap}
          fps={FPS}
          width={WIDTH}
          height={HEIGHT}
        />
        <Composition
          id="Scene11-Outro"
          component={Scene11Outro}
          durationInFrames={SCENE_FRAMES.outro}
          fps={FPS}
          width={WIDTH}
          height={HEIGHT}
        />
      </Folder>
    </>
  );
};
