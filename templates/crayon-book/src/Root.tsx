import React from 'react';
import { Composition } from 'remotion';
import { CrayonBookComposition } from './Composition';

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="Main"
        component={CrayonBookComposition}
        durationInFrames={150}
        fps={30}
        width={1080}
        height={1920}
        defaultProps={{
          title: "THE MOON CAN'T SLEEP",
          storyLine: 'Every night the little stars tell stories, but tonight the moon lost its favorite pillow.',
          characterName: 'LITTLE CLOUD',
          elements: [
            { name: 'STARS', emoji: '⭐' },
            { name: 'PILLOW', emoji: '☁️' },
            { name: 'LANTERN', emoji: '🏮' },
          ],
        }}
      />
    </>
  );
};
