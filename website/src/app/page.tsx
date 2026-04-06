import { motion, useScroll, useTransform } from "framer-motion";
import { Terminal, Zap, Code, Shield } from "lucide-react";
import React from "react";
import Image from "next/image";

const features = [
  {
    title: "Lightweight Engine",
    desc: "Built with Rust for absolute zero overhead and maximum efficiency on modern hardware.",
    icon: Zap,
  },
  {
    title: "Secure by Design",
    desc: "Every command is vetted through our sandboxed execution environment. No surprises.",
    icon: Shield,
  },
  {
    title: "Developer First",
    desc: "Intelligent autocompletion and built-in AI assistance tailored to your workflow.",
    icon: Code,
  },
];

export default function Home() {
  const { scrollYProgress } = useScroll();

  return (
    <main className="relative min-h-screen selection:bg-[#ff1a4a] selection:text-white bg-white text-black overflow-x-hidden font-sans">
      
      {/* Sticky Premium Navbar */}
      <nav className="fixed top-0 w-full z-50 bg-white/70 backdrop-blur-2xl border-b border-black/5 transition-all">
        <div className="max-w-[1500px] mx-auto px-6 h-16 flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div className="w-5 h-5 rounded-[4px] bg-gradient-to-tr from-[#a6e3e9] via-[#ff8cba] to-[#b399ff] border border-black/10 shadow-sm" />
            <span className="font-[800] text-xl tracking-tighter">Bolt</span>
          </div>
          <div className="hidden md:flex gap-8 text-[15px] font-[600] text-gray-500 tracking-tight">
            <a href="#" className="hover:text-black transition-colors">Manifesto</a>
            <a href="#" className="hover:text-black transition-colors">Documentation</a>
            <a href="#" className="hover:text-black transition-colors">Changelog</a>
          </div>
          <div className="flex items-center gap-5">
            <a href="#" className="text-[15px] font-bold text-gray-600 hidden sm:block hover:text-black transition-colors">Sign In</a>
            <button className="px-5 py-2 text-[14px] font-bold bg-black text-white rounded-full hover:scale-105 transition-transform shadow-md tracking-tight">
              Get Started
            </button>
          </div>
        </div>
      </nav>

      {/* Boxed Hero Section */}
      <section className="px-4 md:px-8 max-w-[1500px] mx-auto w-full pt-28 mb-32">
        <div className="relative w-full rounded-[3rem] overflow-hidden py-24 px-6 md:p-24 shadow-[0_20px_50px_-20px_rgba(0,0,0,0.3)] isolate min-h-[85vh] flex flex-col justify-center border border-black/5 group">
          
          {/* Pastel Background Image (grad2.jpg) confined to the box */}
          <div className="absolute inset-0 w-full h-full -z-20 transition-transform duration-1000 group-hover:scale-105">
            <Image 
              src="/grad2.jpg" 
              alt="Bolt Pastel Aesthetic Background" 
              fill 
              className="object-cover scale-[1.02]" 
              priority 
            />
            {/* Subtle vignette/overlay so text remains readable on bright pastels */}
            <div className="absolute inset-0 bg-black/5 mix-blend-multiply" />
            <div className="absolute inset-0 bg-gradient-to-r from-black/20 via-transparent to-transparent" />
          </div>

          {/* Noise Texture strictly inside the Hero Box */}
          <div 
            className="absolute inset-0 -z-10 pointer-events-none mix-blend-soft-light opacity-[0.4]"
            style={{ backgroundImage: `url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='1.5' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E")` }}
          />

          {/* Content Split Layer */}
          <div className="relative z-10 w-full grid lg:grid-cols-12 gap-16 lg:gap-8 items-center max-w-7xl mx-auto">
            
            {/* Left Column: Bold Copy & Design Element */}
            <div className="lg:col-span-7 flex flex-col items-start text-left">
              <motion.div
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.6 }}
                className="flex items-start gap-4 mb-10 origin-left"
              >
                <div className="w-8 h-8 rounded-xl border-2 border-white/20 bg-gradient-to-tr from-[#a6e3e9] via-[#ff8cba] to-[#b399ff] shadow-xl shrink-0 mt-1" />
                <div className="flex flex-col text-white transform -translate-y-1">
                  <span className="text-[1.75rem] font-[900] tracking-tighter leading-none drop-shadow-md">BOLT CLI</span>
                  <span className="text-sm font-[800] text-white/90 uppercase tracking-[0.2em] mt-1.5 drop-shadow-sm">— Game Launcher</span>
                </div>
              </motion.div>

              <motion.h1 
                initial={{ opacity: 0, y: 15 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.1 }}
                className="text-7xl md:text-[7rem] font-[900] text-white leading-[0.9] mb-8 drop-shadow-lg"
                style={{ letterSpacing: "-0.05em" }}
              >
                Zero bloat.<br/>Pure aesthetic.
              </motion.h1>

              <motion.p
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.2 }}
                className="text-white/95 text-xl font-[500] max-w-lg leading-relaxed mb-12 drop-shadow-md tracking-tight"
              >
                Bolt orchestrates your entire library. We built an impossibly performant launcher on top of a next-generation UI architecture.
              </motion.p>

              <motion.div 
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.3 }}
                className="flex flex-wrap items-center gap-6"
              >
                <button className="px-10 py-4 bg-white text-black rounded-full font-[800] text-lg hover:scale-105 hover:bg-gray-50 transition-all duration-300 shadow-[0_20px_40px_-5px_rgba(0,0,0,0.4)] flex items-center gap-3 tracking-tight">
                  <Zap size={22} className="text-black" />
                  Install Bolt
                </button>
                <a href="#" className="font-bold text-white/90 hover:text-white transition-colors underline-offset-4 hover:underline drop-shadow-md tracking-tight">
                  View Documentation
                </a>
              </motion.div>
            </div>

            {/* Right Column: Tweet Embed inside Glassmorphism container */}
            <motion.div 
              initial={{ opacity: 0, scale: 0.95, rotate: 1 }}
              animate={{ opacity: 1, scale: 1, rotate: 0 }}
              transition={{ duration: 0.8, delay: 0.4, type: "spring", stiffness: 100 }}
              className="lg:col-span-5 w-full flex justify-center lg:justify-end"
            >
              <div className="relative w-full max-w-[420px]">
                {/* Floating aesthetic deco dots behind the tweet */}
                <div className="absolute -top-10 -right-10 w-32 h-32 bg-white/20 rounded-full blur-2xl pointer-events-none" />
                <div className="absolute -bottom-10 -left-10 w-32 h-32 bg-white/20 rounded-full blur-2xl pointer-events-none" />
                
                {/* Frosted Glass Container */}
                <div className="relative bg-white/20 backdrop-blur-3xl p-6 rounded-[2.5rem] border border-white/40 shadow-[0_30px_60px_-15px_rgba(0,0,0,0.4)] transform hover:scale-[1.02] transition-transform duration-500">
                  <div className="absolute inset-x-8 top-0 h-px bg-gradient-to-r from-transparent via-white/50 to-transparent" />
                  
                  {/* MacOS-style Header */}
                  <div className="flex gap-2.5 mb-6 px-2 items-center">
                    <div className="w-3 h-3 rounded-full bg-[#ED6A5E] border border-white/20 shadow-sm" />
                    <div className="w-3 h-3 rounded-full bg-[#F4BF4F] border border-white/20 shadow-sm" />
                    <div className="w-3 h-3 rounded-full bg-[#61C554] border border-white/20 shadow-sm" />
                    <span className="ml-auto text-[10px] font-bold tracking-[0.15em] text-white/80 drop-shadow-sm uppercase">Concept Ref</span>
                  </div>
                  
                  {/* The Tweet */}
                  <div className="rounded-2xl overflow-hidden bg-white shadow-inner">
                    <blockquote className="twitter-tweet" data-theme="light">
                      <a href="https://twitter.com/divikkk1/status/2040754364969861307">Loading tweet...</a>
                    </blockquote>
                    <script async src="https://platform.twitter.com/widgets.js" charSet="utf-8"></script>
                  </div>
                </div>
              </div>
            </motion.div>
          </div>
        </div>
      </section>

      {/* Philosophy / Features Section (Clean White) */}
      <section className="px-6 max-w-5xl mx-auto text-center pb-32">
        <motion.p 
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          className="text-gray-400 font-[800] uppercase tracking-[0.2em] text-sm mb-6"
        >
          Built for Gamers
        </motion.p>
        
        <motion.h2 
          initial={{ opacity: 0, y: 10 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          className="text-[2.5rem] md:text-[4rem] font-[800] text-black leading-[1.05] mb-24 drop-shadow-sm"
          style={{ letterSpacing: "-0.04em" }}
        >
          Developers, researchers, and builders<br />
          across different domains to advance Bolt.
        </motion.h2>

        <div className="grid grid-cols-2 lg:grid-cols-4 gap-x-8 gap-y-16 relative px-8 py-12 text-left">
          {/* Subtle borders acting as separators */}
          <div className="absolute top-0 left-0 right-0 h-px bg-black/5" />
          <div className="absolute bottom-0 left-0 right-0 h-px bg-black/5" />
          
          <motion.div 
            initial={{ opacity: 0, y: 10 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.1 }}
            className="flex flex-col items-start group cursor-default"
          >
            <div className="w-16 h-16 rounded-2xl bg-[#f8f9fa] border border-black/5 shadow-sm flex items-center justify-center mb-6 group-hover:bg-[#ebedf0] transition-colors">
              <Terminal className="text-black" strokeWidth={2.5} size={26} />
            </div>
            <p className="text-black font-[800] text-xl tracking-tight leading-tight">Unified<br/>Library</p>
            <p className="text-gray-500 font-medium text-[15px] mt-3 leading-relaxed">One place for all your installed games. No bloat.</p>
          </motion.div>

          <motion.div 
            initial={{ opacity: 0, y: 10 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.2 }}
            className="flex flex-col items-start group cursor-default"
          >
            <div className="w-16 h-16 rounded-2xl bg-[#f8f9fa] border border-black/5 shadow-sm flex items-center justify-center mb-6 group-hover:bg-[#ebedf0] transition-colors">
              <Zap className="text-black" strokeWidth={2.5} size={26} />
            </div>
            <p className="text-black font-[800] text-xl tracking-tight leading-tight">Max FPS<br/>Overlay</p>
            <p className="text-gray-500 font-medium text-[15px] mt-3 leading-relaxed">Kill background tasks instantly during gameplay.</p>
          </motion.div>

          <motion.div 
            initial={{ opacity: 0, y: 10 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.3 }}
            className="flex flex-col items-start group cursor-default"
          >
            <div className="w-16 h-16 rounded-2xl bg-[#f8f9fa] border border-black/5 shadow-sm flex items-center justify-center mb-6 group-hover:bg-[#ebedf0] transition-colors">
              <Shield className="text-black" strokeWidth={2.5} size={26} />
            </div>
            <p className="text-black font-[800] text-xl tracking-tight leading-tight">Seamless<br/>Sync</p>
            <p className="text-gray-500 font-medium text-[15px] mt-3 leading-relaxed">Instant cloud saves backed automatically by Git.</p>
          </motion.div>

          <motion.div 
            initial={{ opacity: 0, y: 10 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.4 }}
            className="flex flex-col items-start group cursor-default"
          >
            <div className="w-16 h-16 rounded-2xl bg-[#f8f9fa] border border-black/5 shadow-sm flex items-center justify-center mb-6 group-hover:bg-[#ebedf0] transition-colors">
              <Code className="text-black" strokeWidth={2.5} size={26} />
            </div>
            <p className="text-black font-[800] text-xl tracking-tight leading-tight">Mod<br/>Manager</p>
            <p className="text-gray-500 font-medium text-[15px] mt-3 leading-relaxed">Click & play complete community patching.</p>
          </motion.div>
        </div>
      </section>
      
      {/* Scroll Progress Bar */}
      <motion.div 
        className="fixed top-0 left-0 right-0 h-[3px] bg-black origin-left z-[100] pointer-events-none"
        style={{ scaleX: scrollYProgress }}
      />
    </main>
  );
}
