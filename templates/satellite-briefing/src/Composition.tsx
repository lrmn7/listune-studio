import React from 'react';
import { interpolate, useCurrentFrame, useVideoConfig } from 'remotion';
import type { SatelliteBriefingProps } from './types';

export const SatelliteBriefingComposition: React.FC<SatelliteBriefingProps> = ({
  missionCodename = 'OPERATION AEGIS ORBIT',
  targetRegion = 'STRAIT OF MALACCA',
  coordinates = '01°25\'48"N 102°49\'12"E',
  altitudeKm = '420 KM',
  status = 'SURVEILLANCE PASS ACTIVE',
  findings = [
    'PRIMARY SHIPPING CORRIDOR: 84% CONGESTION',
    'RADAR REFLECTANCE DEVIATION DETECTED',
    'SURFACE THERMAL SIGNATURE ELEVATED',
    'CONFIRMED SATELLITE PASS ACQUISITION',
  ],
}) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();

  // Scanning line move
  const scanY = (frame * 6) % height;
  const hudProgress = interpolate(frame, [0, 25], [0, 1], { extrapolateRight: 'clamp' });
  const reticlePulse = Math.sin(frame * 0.15) * 0.1 + 0.9;

  return (
    <div
      style={{
        width,
        height,
        backgroundColor: '#080C14',
        color: '#E2E8F0',
        fontFamily: 'Consolas, "Roboto Mono", monospace',
        overflow: 'hidden',
        position: 'relative',
        padding: '60px 50px',
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'space-between',
        boxSizing: 'border-box',
      }}
    >
      {/* Scanline overlay */}
      <div
        style={{
          position: 'absolute',
          top: scanY,
          left: 0,
          right: 0,
          height: 3,
          backgroundColor: '#38BDF8',
          boxShadow: '0 0 15px #38BDF8',
          opacity: 0.6,
          pointerEvents: 'none',
        }}
      />

      {/* Target Reticle in Center */}
      <div
        style={{
          position: 'absolute',
          top: '42%',
          left: '50%',
          width: 300,
          height: 300,
          marginLeft: -150,
          marginTop: -150,
          borderRadius: '50%',
          border: '1px dashed rgba(56, 189, 248, 0.4)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          transform: `scale(${reticlePulse})`,
          pointerEvents: 'none',
        }}
      >
        <div style={{ width: 140, height: 140, borderRadius: '50%', border: '1px solid rgba(245, 158, 11, 0.6)' }} />
        <div style={{ position: 'absolute', width: '100%', height: 1, backgroundColor: 'rgba(56, 189, 248, 0.25)' }} />
        <div style={{ position: 'absolute', height: '100%', width: 1, backgroundColor: 'rgba(56, 189, 248, 0.25)' }} />
      </div>

      {/* Header Bar */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'flex-start',
          borderBottom: '1px solid rgba(56, 189, 248, 0.3)',
          paddingBottom: 20,
          opacity: hudProgress,
        }}
      >
        <div>
          <div
            style={{
              display: 'inline-block',
              backgroundColor: '#F59E0B',
              color: '#080C14',
              padding: '4px 12px',
              fontSize: 14,
              fontWeight: 900,
              letterSpacing: '0.15em',
              marginBottom: 10,
            }}
          >
            TOP SECRET // SATELLITE BRIEFING
          </div>
          <h1
            style={{
              fontSize: 48,
              fontWeight: 800,
              margin: 0,
              letterSpacing: '0.05em',
              color: '#F8FAFC',
            }}
          >
            {missionCodename}
          </h1>
          <div style={{ fontSize: 18, color: '#38BDF8', marginTop: 8, letterSpacing: '0.1em' }}>
            TARGET REGION: {targetRegion}
          </div>
        </div>

        <div style={{ textAlign: 'right', fontSize: 14, color: '#94A3B8' }}>
          <div>ORBIT ALT: {altitudeKm}</div>
          <div style={{ marginTop: 4 }}>UTC TIME: {new Date().toISOString().slice(11, 19)}Z</div>
          <div style={{ color: '#10B981', fontWeight: 700, marginTop: 6 }}>● {status}</div>
        </div>
      </div>

      {/* Telemetry Corner Readout */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          backgroundColor: 'rgba(15, 23, 42, 0.85)',
          border: '1px solid #1E293B',
          padding: '16px 24px',
          borderRadius: 8,
          fontSize: 18,
          letterSpacing: '0.05em',
        }}
      >
        <span>COORD: <strong style={{ color: '#F59E0B' }}>{coordinates}</strong></span>
        <span>OPTICAL RESOLUTION: <strong>0.35M / PX</strong></span>
        <span>BAND: <strong>INFRARED & SAR</strong></span>
      </div>

      {/* Classified Intelligence Dossier Points */}
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          gap: 16,
          backgroundColor: 'rgba(15, 23, 42, 0.7)',
          padding: 24,
          borderLeft: '4px solid #F59E0B',
        }}
      >
        <div style={{ fontSize: 16, fontWeight: 700, letterSpacing: '0.1em', color: '#F59E0B' }}>
          SURVEILLANCE BRIEFING FINDINGS:
        </div>

        {findings.map((item, idx) => {
          const itemProgress = interpolate(frame, [20 + idx * 10, 35 + idx * 10], [0, 1], {
            extrapolateLeft: 'clamp',
            extrapolateRight: 'clamp',
          });

          return (
            <div
              key={idx}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 12,
                fontSize: 18,
                fontWeight: 600,
                opacity: itemProgress,
                transform: `translateX(${(1 - itemProgress) * 20}px)`,
              }}
            >
              <span style={{ color: '#38BDF8' }}>[{idx + 1}]</span>
              <span>{item}</span>
            </div>
          );
        })}
      </div>

      {/* Tactical Footer */}
      <div
        style={{
          borderTop: '1px solid rgba(56, 189, 248, 0.3)',
          paddingTop: 16,
          display: 'flex',
          justifyContent: 'space-between',
          fontSize: 13,
          color: '#64748B',
          letterSpacing: '0.05em',
        }}
      >
        <span>LEMO-OPUSCAR EVIDENCE BRIEFING ENGINE</span>
        <span>SECURITY LEVEL: RESTRICTED // LISTUNE STUDIO</span>
      </div>
    </div>
  );
};
