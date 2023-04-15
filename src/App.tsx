import { Routes, Route } from '@solidjs/router';
import { LedStripConfiguration } from './components/led-strip-configuration/led-strip-configuration';

function App() {
  return (
    <div>
      <div>
        <a href="/led-strips-configuration">灯条配置</a>
        <a href="/white-balance">白平衡</a>
      </div>
      <Routes>
        <Route path="/led-strips-configuration" component={LedStripConfiguration} />
        <Route path="/white-balance" component={LedStripConfiguration} />
      </Routes>
    </div>
  );
}

export default App;
