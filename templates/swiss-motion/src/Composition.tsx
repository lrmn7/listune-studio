import React from 'react';
import { interpolate, useCurrentFrame, useVideoConfig } from 'remotion';
import type { SwissMotionProps } from './types';

// Precise ease curve from Swiss Motion grammar: cubic-bezier(.7, 0, .2, 1)
function cubicBezier(x1: number, y1: number, x2: number, y2: number) {
  const bx = (u: number) => 3 * (1 - u) * (1 - u) * u * x1 + 3 * (1 - u) * u * u * x2 + u * u * u;
  const by = (u: number) => 3 * (1 - u) * (1 - u) * u * y1 + 3 * (1 - u) * u * u * y2 + u * u * u;
  return (t: number) => {
    if (t <= 0) return 0;
    if (t >= 1) return 1;
    let lo = 0;
    let hi = 1;
    let u = t;
    for (let i = 0; i < 20; i++) {
      const x = bx(u);
      if (Math.abs(x - t) < 1e-5) break;
      if (x < t) lo = u;
      else hi = u;
      u = (lo + hi) / 2;
    }
    return by(u);
  };
}

const E = cubicBezier(0.7, 0, 0.2, 1);

export const SwissMotionComposition: React.FC<SwissMotionProps> = ({
  title = 'FIVE RULES FOR A POSTER',
  subtitle = 'International Typographic Style in motion: modular grid, flush-left type, one signal color.',
  leadNumber = '01',
  signalColor = '#E30613',
  points = [
    'Pure 2D graphic design on a modular grid',
    'Every element snaps to grid lines',
    'Extreme scale contrast between type sizes',
    'One exception breaks the system',
  ],
}) => {
  const frame = useCurrentFrame();
  const { fps, width, height } = useVideoConfig();

  // 12-column grid geometry
  const columns = 12;
  const margin = width * 0.05;
  const gutter = width * 0.015;
  const colWidth = (width - margin * 2 - gutter * (columns - 1)) / columns;

  // Animations calculated with Swiss snap easing
  const snap = (startFrame: number, durationFrames = 12) => {
    const raw = interpolate(frame, [startFrame, startFrame + durationFrames], [0, 1], {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });
    return E(raw);
  };

  const numProgress = snap(5, 15);
  const titleProgress = snap(18, 14);
  const subProgress = snap(30, 14);

  return (
    <div
      style={{
        width,
        height,
        backgroundColor: '#EAE9E5',
        color: '#111111',
        fontFamily: 'system-ui, -apple-system, sans-serif',
        overflow: 'hidden',
        position: 'relative',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      {/* 12-column grid lines */}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          pointerEvents: 'none',
          display: 'flex',
          paddingLeft: margin,
          paddingRight: margin,
          gap: gutter,
          opacity: 0.25,
        }}
      >
        {Array.from({ length: columns }).map((_, i) => (
          <div
            key={i}
            style={{
              width: colWidth,
              height: '100%',
              borderLeft: '1px solid #CFCEC9',
              borderRight: '1px solid #CFCEC9',
            }}
          />
        ))}
      </div>

      {/* Header bar / technical indicator */}
      <div
        style={{
          padding: `${margin * 0.6}px ${margin}px`,
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          borderBottom: '1px solid #CFCEC9',
          fontSize: 20,
          fontWeight: 600,
          letterSpacing: '-0.02em',
          textTransform: 'uppercase',
        }}
      >
        <span>SWISS MOTION SYSTEM</span>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <span style={{ fontSize: 16, color: '#77766F' }}>FRAME {frame}</span>
          <div
            style={{
              width: 12,
              height: 12,
              borderRadius: '50%',
              backgroundColor: signalColor,
            }}
          />
        </div>
      </div>

      {/* Main hero spread */}
      <div
        style={{
          flex: 1,
          padding: `${margin}px`,
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'space-between',
          position: 'relative',
        }}
      >
        {/* Giant Numeral */}
        <div
          style={{
            overflow: 'hidden',
            height: '180px',
          }}
        >
          <div
            style={{
              fontSize: '200px',
              fontWeight: 900,
              lineHeight: 0.9,
              letterSpacing: '-0.05em',
              transform: `translateY(${(1 - numProgress) * 100}%)`,
              color: '#111111',
            }}
          >
            {leadNumber}
          </div>
        </div>

        {/* Title with clip-reveal */}
        <div style={{ marginTop: 'auto', marginBottom: 40 }}>
          <div style={{ overflow: 'hidden' }}>
            <h1
              style={{
                fontSize: 68,
                fontWeight: 800,
                lineHeight: 1.05,
                margin: 0,
                letterSpacing: '-0.03em',
                textTransform: 'uppercase',
                transform: `translateY(${(1 - titleProgress) * 100}%)`,
              }}
            >
              {title}
            </h1>
          </div>

          {/* Subtitle */}
          <div style={{ overflow: 'hidden', marginTop: 24 }}>
            <p
              style={{
                fontSize: 28,
                lineHeight: 1.4,
                color: '#444444',
                maxWidth: '85%',
                margin: 0,
                transform: `translateY(${(1 - subProgress) * 100}%)`,
              }}
            >
              {subtitle}
            </p>
          </div>
        </div>

        {/* Modular points list */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(2, 1fr)',
            gap: 20,
            paddingTop: 30,
            borderTop: '2px solid #111111',
          }}
        >
          {points.map((pt, idx) => {
            const pProgress = snap(45 + idx * 8, 12);
            return (
              <div
                key={idx}
                style={{
                  display: 'flex',
                  alignItems: 'baseline',
                  gap: 12,
                  opacity: pProgress,
                  transform: `translateY(${(1 - pProgress) * 20}px)`,
                }}
              >
                <span
                  style={{
                    fontSize: 16,
                    fontWeight: 700,
                    color: idx === 0 ? signalColor : '#111111',
                  }}
                >
                  [{idx + 1}]
                </span>
                <span style={{ fontSize: 20, fontWeight: 500, lineHeight: 1.3 }}>{pt}</span>
              </div>
            );
          })}
        </div>
      </div>

      {/* Bottom status bar */}
      <div
        style={{
          padding: `16px ${margin}px`,
          backgroundColor: '#FFFFFF',
          borderTop: '1px solid #CFCEC9',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          fontSize: 14,
          color: '#77766F',
        }}
      >
        <span>MODULAR GRID: 12 COLUMNS · ARCHIVO / GROTESK</span>
        <div style={{ display: 'flex', gap: 6 }}>
          {Array.from({ length: 8 }).map((_, i) => {
            const active = frame > i * 15;
            return (
              <div
                key={i}
                style={{
                  width: 24,
                  height: 6,
                  backgroundColor: active ? (i === 7 ? signalColor : '#111111') : '#CFCEC9',
                  transition: 'background-color 0.1s',
                }}
              />
            );
          })}
        </div>
      </div>
    </div>
  );
};
