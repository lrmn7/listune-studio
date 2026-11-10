import React from 'react';
import { Composition } from 'remotion';
import { SwissMotionComposition } from './Composition';

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="Main"
        component={SwissMotionComposition}
        durationInFrames={150}
        fps={30}
        width={1080}
        height={1920}
        defaultProps={{
          title: 'FIVE RULES FOR A POSTER',
          subtitle: 'International Typographic Style in motion: modular grid, flush-left type, one signal color.',
          leadNumber: '01',
          signalColor: '#E30613',
          points: [
            'Pure 2D graphic design on a modular grid',
            'Every element snaps to grid lines',
            'Extreme scale contrast between type sizes',
            'One exception breaks the system',
          ],
        }}
      />
    </>
  );
};
