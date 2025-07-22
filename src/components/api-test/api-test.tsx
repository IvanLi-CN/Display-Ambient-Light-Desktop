/**
 * API测试组件
 * 用于测试HTTP API和WebSocket功能
 */

import { createSignal, onMount, For } from 'solid-js';
import { adaptiveApi } from '../../services/api-adapter';
import { LedApiService, ConfigApiService } from '../../services/led-api.service';
import { DisplayApiService, DeviceApiService, InfoApiService } from '../../services/display-api.service';
import { WebSocketListener, WebSocketEventHandlers } from '../websocket-listener';
import { useLanguage } from '../../i18n';

interface TestResult {
  name: string;
  status: 'pending' | 'success' | 'error';
  message: string;
  duration?: number;
}

export const ApiTest = () => {
  const { t } = useLanguage();
  const [testResults, setTestResults] = createSignal<TestResult[]>([]);
  const [isRunning, setIsRunning] = createSignal(false);
  const [environmentInfo, setEnvironmentInfo] = createSignal<any>(null);

  // WebSocket事件处理器
  const wsHandlers: WebSocketEventHandlers = {
    onLedColorsChanged: (data) => {
      console.log('收到LED颜色变化事件:', data);
      addTestResult('WebSocket LED颜色事件', 'success', '收到实时LED颜色更新');
    },
    onConfigChanged: (data) => {
      console.log('收到配置变化事件:', data);
      addTestResult('WebSocket 配置事件', 'success', '收到配置更新通知');
    },
    onConnectionStatusChanged: (connected) => {
      console.log('WebSocket连接状态:', connected);
      addTestResult('WebSocket 连接', connected ? 'success' : 'error', 
        connected ? '已连接到WebSocket' : 'WebSocket连接断开');
    }
  };

  // 添加测试结果
  const addTestResult = (name: string, status: TestResult['status'], message: string, duration?: number) => {
    setTestResults(prev => [...prev, { name, status, message, duration }]);
  };

  // 运行单个测试
  const runTest = async (name: string, testFn: () => Promise<any>) => {
    const startTime = Date.now();
    addTestResult(name, 'pending', '测试中...');
    
    try {
      const result = await testFn();
      const duration = Date.now() - startTime;
      
      setTestResults(prev => prev.map(test => 
        test.name === name && test.status === 'pending'
          ? { ...test, status: 'success', message: `成功 (${JSON.stringify(result).slice(0, 100)}...)`, duration }
          : test
      ));
    } catch (error) {
      const duration = Date.now() - startTime;
      
      setTestResults(prev => prev.map(test => 
        test.name === name && test.status === 'pending'
          ? { ...test, status: 'error', message: `失败: ${error}`, duration }
          : test
      ));
    }
  };

  // 运行所有API测试
  const runAllTests = async () => {
    setIsRunning(true);
    setTestResults([]);

    // 环境检测测试
    await runTest('环境检测', async () => {
      const info = await adaptiveApi.initialize();
      setEnvironmentInfo(info);
      return info;
    });

    // 基础API测试
    await runTest('问候API', () => adaptiveApi.greet('API测试'));
    
    await runTest('应用版本', () => InfoApiService.getAppVersion());
    
    await runTest('显示器信息', () => DisplayApiService.listDisplayInfo());
    
    await runTest('设备列表', () => DeviceApiService.getBoards());

    // LED API测试
    await runTest('LED配置读取', () => ConfigApiService.readLedStripConfigs());
    
    await runTest('测试模式状态', () => LedApiService.isTestModeActive());

    // WebSocket测试
    await runTest('WebSocket消息发送', async () => {
      adaptiveApi.emitEvent('test_message', { message: 'Hello from frontend!' });
      return 'WebSocket消息已发送';
    });

    setIsRunning(false);
  };

  // 清空测试结果
  const clearResults = () => {
    setTestResults([]);
  };

  // 组件挂载时获取环境信息
  onMount(async () => {
    try {
      const info = await adaptiveApi.initialize();
      setEnvironmentInfo(info);
    } catch (error) {
      console.error('初始化API适配器失败:', error);
    }
  });

  return (
    <div class="api-test-container">
      <div class="header">
        <h2 class="text-2xl font-bold mb-4">API测试工具</h2>
        <p class="text-base-content/70 mb-6">
          测试HTTP API和WebSocket功能，验证前端与后端的通信
        </p>
      </div>

      {/* 环境信息 */}
      {environmentInfo() && (
        <div class="environment-info mb-6">
          <h3 class="text-lg font-semibold mb-2">运行环境</h3>
          <div class="bg-base-200 p-4 rounded-lg">
            <div class="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span class="font-medium">Tauri环境:</span>
                <span class={`ml-2 ${environmentInfo().isTauri ? 'text-success' : 'text-warning'}`}>
                  {environmentInfo().isTauri ? '✓ 是' : '✗ 否'}
                </span>
              </div>
              <div>
                <span class="font-medium">HTTP API:</span>
                <span class={`ml-2 ${environmentInfo().isHttpApiAvailable ? 'text-success' : 'text-error'}`}>
                  {environmentInfo().isHttpApiAvailable ? '✓ 可用' : '✗ 不可用'}
                </span>
              </div>
              <div class="col-span-2">
                <span class="font-medium">首选模式:</span>
                <span class="ml-2 font-mono bg-base-300 px-2 py-1 rounded">
                  {environmentInfo().preferredMode}
                </span>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* WebSocket状态 */}
      <div class="websocket-section mb-6">
        <h3 class="text-lg font-semibold mb-2">WebSocket连接</h3>
        <WebSocketListener handlers={wsHandlers} showStatus={true} />
      </div>

      {/* 测试控制 */}
      <div class="test-controls mb-6">
        <div class="flex gap-4">
          <button 
            class="btn btn-primary"
            onClick={runAllTests}
            disabled={isRunning()}
          >
            {isRunning() ? '测试中...' : '运行所有测试'}
          </button>
          
          <button 
            class="btn btn-outline"
            onClick={clearResults}
            disabled={isRunning()}
          >
            清空结果
          </button>
        </div>
      </div>

      {/* 测试结果 */}
      <div class="test-results">
        <h3 class="text-lg font-semibold mb-4">测试结果</h3>
        
        {testResults().length === 0 ? (
          <div class="text-base-content/50 text-center py-8">
            点击"运行所有测试"开始测试
          </div>
        ) : (
          <div class="space-y-2">
            <For each={testResults()}>
              {(result) => (
                <div class={`test-result-item p-4 rounded-lg border-l-4 ${
                  result.status === 'success' ? 'bg-success/10 border-success' :
                  result.status === 'error' ? 'bg-error/10 border-error' :
                  'bg-warning/10 border-warning'
                }`}>
                  <div class="flex justify-between items-start">
                    <div class="flex-1">
                      <div class="font-medium">{result.name}</div>
                      <div class="text-sm text-base-content/70 mt-1">
                        {result.message}
                      </div>
                    </div>
                    <div class="flex items-center gap-2 text-sm">
                      {result.duration && (
                        <span class="text-base-content/50">
                          {result.duration}ms
                        </span>
                      )}
                      <span class={`status-badge ${
                        result.status === 'success' ? 'text-success' :
                        result.status === 'error' ? 'text-error' :
                        'text-warning'
                      }`}>
                        {result.status === 'success' ? '✓' :
                         result.status === 'error' ? '✗' : '⏳'}
                      </span>
                    </div>
                  </div>
                </div>
              )}
            </For>
          </div>
        )}
      </div>

      <style>{`
        .api-test-container {
          max-width: 800px;
          margin: 0 auto;
          padding: 2rem;
        }

        .status-badge {
          font-weight: bold;
          font-size: 1.2em;
        }
      `}</style>
    </div>
  );
};
