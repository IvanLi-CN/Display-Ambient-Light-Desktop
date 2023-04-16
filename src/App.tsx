import { Routes, Route } from '@solidjs/router';
import { LedStripConfiguration } from './components/led-strip-configuration/led-strip-configuration';
import { WhiteBalance } from './components/white-balance/white-balance';

function App() {
  return (
    <div>
      <div>
        <a href="/led-strips-configuration">灯条配置</a>
        <a href="/white-balance">白平衡</a>
      </div>
      <Routes>
        <Route path="/led-strips-configuration" component={LedStripConfiguration} />
        <Route path="/white-balance" component={WhiteBalance} />
      </Routes>
    </div>
  );
}

export default App;
