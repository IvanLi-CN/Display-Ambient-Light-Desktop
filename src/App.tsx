import { Routes, Route } from '@solidjs/router';
import { LedStripConfiguration } from './components/led-strip-configuration/led-strip-configuration';
import { WhiteBalance } from './components/white-balance/white-balance';
import { createEffect } from 'solid-js';
import { invoke } from '@tauri-apps/api';
import { setLedStripStore } from './stores/led-strip.store';
import { LedStripConfigContainer } from './models/led-strip-config';
import { InfoIndex } from './components/info/info-index';
import { DisplayStateIndex } from './components/displays/display-state-index';

function App() {
  createEffect(() => {
    invoke<LedStripConfigContainer>('read_config').then((config) => {
      console.log('read config', config);
      setLedStripStore({
        strips: config.strips,
        mappers: config.mappers,
        colorCalibration: config.color_calibration,
      });
    });
  });

  return (
    <div>
      <div>
        <a href="/info">基本信息</a>
        <a href="/displays">显示器信息</a>
        <a href="/led-strips-configuration">灯条配置</a>
        <a href="/white-balance">白平衡</a>
      </div>
      <Routes>
        <Route path="/info" component={InfoIndex} />
        <Route path="/displays" component={DisplayStateIndex} />
        <Route path="/led-strips-configuration" component={LedStripConfiguration} />
        <Route path="/white-balance" component={WhiteBalance} />
      </Routes>
    </div>
  );
}

export default App;
