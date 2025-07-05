import { Routes, Route } from '@solidjs/router';
import { LedStripConfiguration } from './components/led-strip-configuration/led-strip-configuration';
import { WhiteBalance } from './components/white-balance/white-balance';
import { createEffect } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { setLedStripStore } from './stores/led-strip.store';
import { LedStripConfigContainer } from './models/led-strip-config';
import { InfoIndex } from './components/info/info-index';
import { DisplayStateIndex } from './components/displays/display-state-index';

function App() {
  createEffect(() => {
    invoke<LedStripConfigContainer>('read_config').then((config) => {
      setLedStripStore({
        strips: config.strips,
        mappers: config.mappers,
        colorCalibration: config.color_calibration,
      });
    }).catch((error) => {
      console.error('Failed to read config:', error);
    });
  });

  return (
    <div class="min-h-screen bg-base-100" data-theme="dark">
      {/* Navigation */}
      <div class="navbar bg-base-200 shadow-lg">
        <div class="navbar-start">
          <div class="dropdown">
            <div tabindex="0" role="button" class="btn btn-ghost lg:hidden">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h8m-8 6h16"></path>
              </svg>
            </div>
            <ul tabindex="0" class="menu menu-sm dropdown-content mt-3 z-[1] p-2 shadow bg-base-100 rounded-box w-52">
              <li><a href="/info" class="text-base-content">基本信息</a></li>
              <li><a href="/displays" class="text-base-content">显示器信息</a></li>
              <li><a href="/led-strips-configuration" class="text-base-content">灯条配置</a></li>
              <li><a href="/white-balance" class="text-base-content">白平衡</a></li>
            </ul>
          </div>
          <a class="btn btn-ghost text-xl text-primary font-bold">环境光控制</a>
        </div>
        <div class="navbar-center hidden lg:flex">
          <ul class="menu menu-horizontal px-1">
            <li><a href="/info" class="btn btn-ghost text-base-content hover:text-primary">基本信息</a></li>
            <li><a href="/displays" class="btn btn-ghost text-base-content hover:text-primary">显示器信息</a></li>
            <li><a href="/led-strips-configuration" class="btn btn-ghost text-base-content hover:text-primary">灯条配置</a></li>
            <li><a href="/white-balance" class="btn btn-ghost text-base-content hover:text-primary">白平衡</a></li>
          </ul>
        </div>
        <div class="navbar-end">
          <div class="badge badge-primary badge-outline">v1.0</div>
        </div>
      </div>

      {/* Main Content */}
      <main class="container mx-auto p-4">
        <Routes>
          <Route path="/info" component={InfoIndex} />
          <Route path="/displays" component={DisplayStateIndex} />
          <Route path="/led-strips-configuration" component={LedStripConfiguration} />
          <Route path="/white-balance" component={WhiteBalance} />
        </Routes>
      </main>
    </div>
  );
}

export default App;
