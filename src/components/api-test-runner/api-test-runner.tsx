/**
 * API测试运行器组件
 * 在浏览器中运行API集成测试并显示结果
 */

import { createSignal, For, Show } from 'solid-js';
import { apiTester, TestResult } from '../../tests/api-integration.test';
import { useLanguage } from '../../i18n';

export function ApiTestRunner() {
  const { t } = useLanguage();
  const [isRunning, setIsRunning] = createSignal(false);
  const [testResults, setTestResults] = createSignal<TestResult[]>([]);
  const [currentTest, setCurrentTest] = createSignal<string>('');

  // 运行所有测试
  const runAllTests = async () => {
    setIsRunning(true);
    setTestResults([]);
    setCurrentTest('正在初始化测试...');

    try {
      // 创建一个自定义的测试器实例来监听进度
      const results = await apiTester.runAllTests();
      setTestResults(results);
      setCurrentTest('测试完成');
    } catch (error) {
      console.error('测试运行失败:', error);
      setCurrentTest('测试运行失败');
    } finally {
      setIsRunning(false);
    }
  };

  // 清空测试结果
  const clearResults = () => {
    setTestResults([]);
    setCurrentTest('');
  };

  // 获取状态图标
  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'success':
        return '✅';
      case 'error':
        return '❌';
      case 'warning':
        return '⚠️';
      default:
        return '❓';
    }
  };

  // 获取状态颜色类
  const getStatusColorClass = (status: string) => {
    switch (status) {
      case 'success':
        return 'text-success';
      case 'error':
        return 'text-error';
      case 'warning':
        return 'text-warning';
      default:
        return 'text-base-content';
    }
  };

  // 计算统计信息
  const getStats = () => {
    const results = testResults();
    const total = results.length;
    const success = results.filter(r => r.status === 'success').length;
    const errors = results.filter(r => r.status === 'error').length;
    const warnings = results.filter(r => r.status === 'warning').length;
    const successRate = total > 0 ? ((success / total) * 100).toFixed(1) : '0';

    return { total, success, errors, warnings, successRate };
  };

  return (
    <div class="container mx-auto p-6 max-w-6xl">
      <div class="card bg-base-100 shadow-xl">
        <div class="card-body">
          <h2 class="card-title text-2xl mb-4">
            🧪 API集成测试
          </h2>
          
          <p class="text-base-content/70 mb-6">
            此工具会测试前端API调用与后端实现的一致性，帮助识别和修复接口问题。
          </p>

          {/* 控制按钮 */}
          <div class="flex gap-4 mb-6">
            <button
              class="btn btn-primary"
              onClick={runAllTests}
              disabled={isRunning()}
            >
              <Show when={isRunning()}>
                <span class="loading loading-spinner loading-sm"></span>
              </Show>
              {isRunning() ? '测试运行中...' : '运行所有测试'}
            </button>
            
            <button
              class="btn btn-outline"
              onClick={clearResults}
              disabled={isRunning()}
            >
              清空结果
            </button>
          </div>

          {/* 当前测试状态 */}
          <Show when={isRunning()}>
            <div class="alert alert-info mb-6">
              <span class="loading loading-spinner loading-sm"></span>
              <span>{currentTest()}</span>
            </div>
          </Show>

          {/* 测试统计 */}
          <Show when={testResults().length > 0}>
            <div class="stats shadow mb-6 w-full">
              <div class="stat">
                <div class="stat-title">总测试数</div>
                <div class="stat-value">{getStats().total}</div>
              </div>
              
              <div class="stat">
                <div class="stat-title">成功</div>
                <div class="stat-value text-success">{getStats().success}</div>
              </div>
              
              <div class="stat">
                <div class="stat-title">失败</div>
                <div class="stat-value text-error">{getStats().errors}</div>
              </div>
              
              <div class="stat">
                <div class="stat-title">警告</div>
                <div class="stat-value text-warning">{getStats().warnings}</div>
              </div>
              
              <div class="stat">
                <div class="stat-title">成功率</div>
                <div class="stat-value">{getStats().successRate}%</div>
              </div>
            </div>
          </Show>

          {/* 测试结果列表 */}
          <Show when={testResults().length > 0}>
            <div class="overflow-x-auto">
              <table class="table table-zebra w-full">
                <thead>
                  <tr>
                    <th>状态</th>
                    <th>测试名称</th>
                    <th>端点</th>
                    <th>方法</th>
                    <th>响应时间</th>
                    <th>状态码</th>
                    <th>消息</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={testResults()}>
                    {(result) => (
                      <tr class={result.status === 'error' ? 'bg-error/10' : ''}>
                        <td>
                          <span class="text-lg">
                            {getStatusIcon(result.status)}
                          </span>
                        </td>
                        <td class="font-medium">{result.name}</td>
                        <td class="font-mono text-sm">{result.endpoint}</td>
                        <td>
                          <span class="badge badge-outline badge-sm">
                            {result.method}
                          </span>
                        </td>
                        <td class="text-sm">
                          {result.responseTime ? `${result.responseTime}ms` : '-'}
                        </td>
                        <td>
                          <Show when={result.statusCode}>
                            <span class={`badge badge-sm ${
                              result.statusCode! >= 200 && result.statusCode! < 300 
                                ? 'badge-success' 
                                : result.statusCode! >= 400 
                                  ? 'badge-error' 
                                  : 'badge-warning'
                            }`}>
                              {result.statusCode}
                            </span>
                          </Show>
                        </td>
                        <td class={`text-sm ${getStatusColorClass(result.status)}`}>
                          {result.message}
                        </td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </div>
          </Show>

          {/* 空状态 */}
          <Show when={testResults().length === 0 && !isRunning()}>
            <div class="text-center py-12">
              <div class="text-6xl mb-4">🧪</div>
              <h3 class="text-xl font-semibold mb-2">准备运行API测试</h3>
              <p class="text-base-content/70">
                点击"运行所有测试"按钮开始测试所有API端点
              </p>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
}
