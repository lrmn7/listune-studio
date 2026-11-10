import React from 'react';
import { FluidMeshBackground } from './components/FluidMeshBackground';
import { Header } from './components/Header';
import { Hero } from './components/Hero';
import { InstallSection } from './components/InstallSection';
import { FeaturesGrid } from './components/FeaturesGrid';
import { FAQSection } from './components/FAQSection';
import { Footer } from './components/Footer';

export default function App() {
  return (
    <div className="relative min-h-screen text-[#f5f6ff] flex flex-col justify-between selection:bg-[#6f9dff]/30">
      {/* Dynamic moving mesh gradient background */}
      <FluidMeshBackground />

      {/* Foreground contents */}
      <div className="flex-1">
        <Header />
        <main>
          <Hero />
          <InstallSection />
          <FeaturesGrid />
          <FAQSection />
        </main>
      </div>

      <Footer />
    </div>
  );
}

