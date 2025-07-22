import { createSignal, onMount } from 'solid-js';
import { adaptiveApi } from '../../services/api-adapter';
import { LedApiService } from '../../services/led-api.service';
import { DataSendMode } from '../../models/led-data-sender';

export const LedDataSenderTest = () => {
  const [currentMode, setCurrentMode] = createSignal<DataSendMode>(DataSendMode.None);
  const [testResult, setTestResult] = createSignal<string>('');
  const [isLoading, setIsLoading] = createSignal(false);

  // 获取当前模式
  const getCurrentMode = async () => {
    try {
      const mode = await LedApiService.getDataSendMode();
      setCurrentMode(mode);
    } catch (error) {
      console.error('Failed to get current mode:', error);
    }
  };

  // 设置模式
  const setMode = async (mode: DataSendMode) => {
    try {
      setIsLoading(true);
      await LedApiService.setDataSendMode(mode);
      setCurrentMode(mode);
      console.log(`Mode set to: ${mode}`);
    } catch (error) {
      console.error('Failed to set mode:', error);
    } finally {
      setIsLoading(false);
    }
  };

  // 运行测试
  const runTest = async () => {
    try {
      setIsLoading(true);
      const result = await LedApiService.testLedDataSender();
      setTestResult(result);
    } catch (error) {
      console.error('Failed to run test:', error);
      setTestResult(`Test failed: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  onMount(() => {
    getCurrentMode();
  });

  return (
    <div class="p-6 max-w-4xl mx-auto">
      <h1 class="text-2xl font-bold mb-6">LED数据发送器测试</h1>
      
      {/* 当前模式显示 */}
      <div class="card bg-base-100 shadow-md mb-6">
        <div class="card-body">
          <h2 class="card-title">当前模式</h2>
          <div class="flex items-center gap-4">
            <span class="text-lg">
              模式: <span class="badge badge-primary">{currentMode()}</span>
            </span>
            <button 
              class="btn btn-sm btn-outline"
              onClick={getCurrentMode}
              disabled={isLoading()}
            >
              刷新
            </button>
          </div>
        </div>
      </div>

      {/* 模式切换 */}
      <div class="card bg-base-100 shadow-md mb-6">
        <div class="card-body">
          <h2 class="card-title">模式切换</h2>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            {Object.values(DataSendMode).map((mode) => (
              <button
                class={`btn ${currentMode() === mode ? 'btn-primary' : 'btn-outline'}`}
                onClick={() => setMode(mode)}
                disabled={isLoading()}
              >
                {mode}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* 测试功能 */}
      <div class="card bg-base-100 shadow-md mb-6">
        <div class="card-body">
          <h2 class="card-title">功能测试</h2>
          <div class="flex gap-4">
            <button
              class="btn btn-secondary"
              onClick={runTest}
              disabled={isLoading()}
            >
              {isLoading() ? '测试中...' : '运行测试'}
            </button>
          </div>
          
          {testResult() && (
            <div class="mt-4">
              <h3 class="font-semibold mb-2">测试结果:</h3>
              <pre class="bg-base-200 p-4 rounded text-sm whitespace-pre-wrap">
                {testResult()}
              </pre>
            </div>
          )}
        </div>
      </div>

      {/* 使用说明 */}
      <div class="card bg-base-100 shadow-md">
        <div class="card-body">
          <h2 class="card-title">使用说明</h2>
          <div class="space-y-2 text-sm">
            <p><strong>None:</strong> 不发送任何数据，用于暂停所有LED数据传输</p>
            <p><strong>AmbientLight:</strong> 屏幕氛围光模式，发送基于屏幕内容的颜色数据</p>
            <p><strong>StripConfig:</strong> 单灯条配置模式，用于灯条配置和测试</p>
            <p><strong>TestEffect:</strong> 测试效果模式，发送预定义的测试动画数据</p>
          </div>
        </div>
      </div>
    </div>
  );
};
