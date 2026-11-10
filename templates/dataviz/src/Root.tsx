import React from 'react';
import { Composition } from 'remotion';
import { DatavizComposition } from './Composition';

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="Main"
        component={DatavizComposition}
        durationInFrames={150}
        fps={30}
        width={1080}
        height={1920}
        defaultProps={{
          title: 'GLOBAL AI COMPUTE DEMAND',
          headlineMetric: '4.8x',
          metricLabel: 'SURGE OVER 18 MONTHS',
          sourceNote: 'SOURCE: AI METRICS INDEX 2026',
          data: [
            { label: 'DATA CENTERS', value: 92, delta: '+140%', highlight: true },
            { label: 'EDGE INFERENCE', value: 68, delta: '+85%' },
            { label: 'AUTONOMOUS FLEETS', value: 54, delta: '+62%' },
            { label: 'CONSUMER APPS', value: 41, delta: '+38%' },
          ],
        }}
      />
    </>
  );
};
