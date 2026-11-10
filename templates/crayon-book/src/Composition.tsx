import React from 'react';
import { interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import type { CrayonBookProps } from './types';

export const CrayonBookComposition: React.FC<CrayonBookProps> = ({
  title = "THE MOON CAN'T SLEEP",
  storyLine = 'Every night the little stars tell stories, but tonight the moon lost its favorite pillow.',
  characterName = 'LITTLE CLOUD',
  elements = [
    { name: 'STARS', emoji: '⭐' },
    { name: 'PILLOW', emoji: '☁️' },
    { name: 'LANTERN', emoji: '🏮' },
  ],
}) => {
  const frame = useCurrentFrame();
  const { fps, width, height } = useVideoConfig();

  // Subtle organic jitter on twos (every 2 frames)
  const jitterFrame = Math.floor(frame / 2);
  const jitterX = Math.sin(jitterFrame * 12.3) * 2;
  const jitterY = Math.cos(jitterFrame * 8.7) * 2;
  const jitterRot = Math.sin(jitterFrame * 4.5) * 0.8;

  // Spring animation for moon pop-in
  const moonScale = spring({
    frame: frame - 10,
    fps,
    config: { damping: 12, mass: 0.8, stiffness: 100 },
  });

  const textFade = interpolate(frame, [30, 50], [0, 1], { extrapolateRight: 'clamp' });

  return (
    <div
      style={{
        width,
        height,
        backgroundColor: '#FBF7EE',
        color: '#264653',
        fontFamily: '"Comic Sans MS", "Chalkboard SE", "Architects Daughter", cursive, sans-serif',
        overflow: 'hidden',
        position: 'relative',
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'space-between',
        padding: '100px 70px',
        boxSizing: 'border-box',
        transform: `translate(${jitterX}px, ${jitterY}px) rotate(${jitterRot}deg)`,
      }}
    >
      {/* Crayon Paper Grain Texture */}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          opacity: 0.12,
          backgroundImage: 'radial-gradient(#264653 1px, transparent 1px)',
          backgroundSize: '16px 16px',
          pointerEvents: 'none',
        }}
      />

      {/* Hand-drawn Outer Crayon Frame */}
      <div
        style={{
          position: 'absolute',
          inset: 40,
          border: '5px dashed #E76F51',
          borderRadius: 30,
          opacity: 0.7,
          pointerEvents: 'none',
        }}
      />

      {/* Storybook Header */}
      <div>
        <div
          style={{
            display: 'inline-block',
            backgroundColor: '#E9C46A',
            color: '#264653',
            padding: '8px 20px',
            borderRadius: '24px',
            fontSize: 22,
            fontWeight: 800,
            transform: 'rotate(-2deg)',
            boxShadow: '3px 4px 0px #2A9D8F',
          }}
        >
          CHAPTER 1 · {characterName}
        </div>

        <h1
          style={{
            fontSize: 64,
            fontWeight: 900,
            lineHeight: 1.1,
            color: '#E76F51',
            marginTop: 30,
            marginBottom: 0,
            letterSpacing: '-0.02em',
            textShadow: '3px 4px 0px #F4A261',
          }}
        >
          {title}
        </h1>
      </div>

      {/* Main Illustration Area: The Moon & Stars */}
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          position: 'relative',
          margin: '40px 0',
        }}
      >
        {/* Animated Moon Illustration */}
        <div
          style={{
            width: 320,
            height: 320,
            borderRadius: '50%',
            backgroundColor: '#F4A261',
            border: '8px solid #264653',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            boxShadow: '8px 10px 0px #E76F51',
            transform: `scale(${Math.max(0, moonScale)}) rotate(${Math.sin(frame * 0.08) * 6}deg)`,
            position: 'relative',
          }}
        >
          {/* Moon Face */}
          <div style={{ display: 'flex', gap: 60, marginBottom: 20 }}>
            {/* Left Eye */}
            <div
              style={{
                width: 24,
                height: 32,
                borderRadius: '50%',
                backgroundColor: '#264653',
                transform: 'rotate(10deg)',
              }}
            />
            {/* Right Eye */}
            <div
              style={{
                width: 24,
                height: 32,
                borderRadius: '50%',
                backgroundColor: '#264653',
                transform: 'rotate(-10deg)',
              }}
            />
          </div>

          {/* Sleepy Mouth */}
          <div
            style={{
              width: 50,
              height: 25,
              borderRadius: '0 0 50px 50px',
              border: '6px solid #264653',
              borderTop: 'none',
            }}
          />

          {/* Rosy Cheeks */}
          <div
            style={{
              position: 'absolute',
              left: 40,
              top: 170,
              width: 35,
              height: 20,
              borderRadius: '50%',
              backgroundColor: '#E76F51',
              opacity: 0.6,
            }}
          />
          <div
            style={{
              position: 'absolute',
              right: 40,
              top: 170,
              width: 35,
              height: 20,
              borderRadius: '50%',
              backgroundColor: '#E76F51',
              opacity: 0.6,
            }}
          />
        </div>

        {/* Floating Story Elements */}
        <div style={{ display: 'flex', gap: 30, marginTop: 40 }}>
          {elements.map((el, i) => {
            const elScale = spring({
              frame: frame - (40 + i * 10),
              fps,
              config: { damping: 10, stiffness: 120 },
            });
            return (
              <div
                key={i}
                style={{
                  backgroundColor: '#FFFFFF',
                  border: '4px solid #264653',
                  borderRadius: 20,
                  padding: '12px 24px',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 10,
                  fontSize: 22,
                  fontWeight: 800,
                  boxShadow: '4px 4px 0px #2A9D8F',
                  transform: `scale(${Math.max(0, elScale)}) rotate(${i % 2 === 0 ? 3 : -3}deg)`,
                }}
              >
                <span style={{ fontSize: 28 }}>{el.emoji}</span>
                <span>{el.name}</span>
              </div>
            );
          })}
        </div>
      </div>

      {/* Storybook Subtitle / Narration Card */}
      <div
        style={{
          backgroundColor: '#FFFFFF',
          border: '4px solid #264653',
          borderRadius: 24,
          padding: 30,
          boxShadow: '6px 8px 0px #264653',
          opacity: textFade,
          transform: `translateY(${(1 - textFade) * 30}px)`,
        }}
      >
        <p
          style={{
            fontSize: 28,
            lineHeight: 1.5,
            margin: 0,
            fontWeight: 700,
            color: '#264653',
          }}
        >
          {storyLine}
        </p>
      </div>
    </div>
  );
};
