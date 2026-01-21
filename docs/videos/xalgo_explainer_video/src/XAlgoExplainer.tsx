import { AbsoluteFill, Series } from "remotion";
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

type SceneFrames = {
  hook: number;
  bigPicture: number;
  request: number;
  thunder: number;
  phoenix: number;
  filtering: number;
  scoring: number;
  weightedScoring: number;
  selection: number;
  recap: number;
  outro: number;
};

type XAlgoExplainerProps = {
  sceneFrames: SceneFrames;
};

export const XAlgoExplainer: React.FC<XAlgoExplainerProps> = ({ sceneFrames }) => {
  return (
    <AbsoluteFill className="bg-x-dark">
      <Series>
        <Series.Sequence durationInFrames={sceneFrames.hook}>
          <Scene1Hook />
        </Series.Sequence>
        <Series.Sequence durationInFrames={sceneFrames.bigPicture}>
          <Scene2BigPicture />
        </Series.Sequence>
        <Series.Sequence durationInFrames={sceneFrames.request}>
          <Scene3Request />
        </Series.Sequence>
        <Series.Sequence durationInFrames={sceneFrames.thunder}>
          <Scene4Thunder />
        </Series.Sequence>
        <Series.Sequence durationInFrames={sceneFrames.phoenix}>
          <Scene5Phoenix />
        </Series.Sequence>
        <Series.Sequence durationInFrames={sceneFrames.filtering}>
          <Scene6Filtering />
        </Series.Sequence>
        <Series.Sequence durationInFrames={sceneFrames.scoring}>
          <Scene7Scoring />
        </Series.Sequence>
        <Series.Sequence durationInFrames={sceneFrames.weightedScoring}>
          <Scene8WeightedScoring />
        </Series.Sequence>
        <Series.Sequence durationInFrames={sceneFrames.selection}>
          <Scene9Selection />
        </Series.Sequence>
        <Series.Sequence durationInFrames={sceneFrames.recap}>
          <Scene10Recap />
        </Series.Sequence>
        <Series.Sequence durationInFrames={sceneFrames.outro}>
          <Scene11Outro />
        </Series.Sequence>
      </Series>
    </AbsoluteFill>
  );
};
