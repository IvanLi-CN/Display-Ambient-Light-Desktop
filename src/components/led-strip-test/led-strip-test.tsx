import { createSignal, createEffect, For, Show, onCleanup, onMount } from 'solid-js';
import { adaptiveApi } from '../../services/api-adapter';
import { LedApiService } from '../../services/led-api.service';
import { DeviceApiService } from '../../services/display-api.service';
import { useLanguage } from '../../i18n/index';
import { LedPreview } from '../led-preview/led-preview';

interface BoardInfo {
  fullname: string;
  host: string;
  address: string;
  port: number;
  connect_status: 'Connected' | 'Disconnected' | { Connecting: number };
}

interface TestPattern {
  name: string;
  description: string;
  effect_type: string;
}

interface TestEffectConfig {
  effect_type: string;
  led_count: number;
  led_type: string;
  speed: number;
  offset: number;
}

export const LedStripTest = () => {
  const { t } = useLanguage();
  const [boards, setBoards] = createSignal<BoardInfo[]>([]);
  const [selectedBoard, setSelectedBoard] = createSignal<BoardInfo | null>(null);
  const [ledCount, setLedCount] = createSignal(60);
  const [ledType, setLedType] = createSignal<'WS2812B' | 'SK6812'>('WS2812B');
  const [ledOffset, setLedOffset] = createSignal(0);
  const [isRunning, setIsRunning] = createSignal(false);
  const [currentPattern, setCurrentPattern] = createSignal<TestPattern | null>(null);
  const [animationSpeed, setAnimationSpeed] = createSignal(33); // ~30fps

  // Temporary input values for better UX
  const [ledCountInput, setLedCountInput] = createSignal('60');
  const [ledOffsetInput, setLedOffsetInput] = createSignal('0');
  const [animationSpeedInput, setAnimationSpeedInput] = createSignal('33');

  // Input handlers for LED count
  const handleLedCountInput = (e: Event) => {
    const target = e.target as HTMLInputElement;
    setLedCountInput(target.value);
  };

  const handleLedCountBlur = (e: Event) => {
    const target = e.target as HTMLInputElement;
    const value = parseInt(target.value);
    if (!isNaN(value) && value >= 1 && value <= 1000) {
      setLedCount(value);
      setLedCountInput(value.toString());
    } else {
      // Reset to current valid value
      setLedCountInput(ledCount().toString());
      target.value = ledCount().toString();
    }
  };

  const handleLedCountKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleLedCountBlur(e);
    }
  };

  // Input handlers for animation speed
  const handleAnimationSpeedInput = (e: Event) => {
    const target = e.target as HTMLInputElement;
    setAnimationSpeedInput(target.value);
  };

  const handleAnimationSpeedBlur = (e: Event) => {
    const target = e.target as HTMLInputElement;
    const value = parseInt(target.value);
    if (!isNaN(value) && value >= 16 && value <= 3600000) { // Max 1 hour (3600000ms)
      setAnimationSpeed(value);
      setAnimationSpeedInput(value.toString());
    } else {
      // Reset to current valid value
      setAnimationSpeedInput(animationSpeed().toString());
      target.value = animationSpeed().toString();
    }
  };

  const handleAnimationSpeedKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleAnimationSpeedBlur(e);
    }
  };

  // Input handlers for LED offset
  const handleLedOffsetInput = (e: Event) => {
    const target = e.target as HTMLInputElement;
    setLedOffsetInput(target.value);
  };

  const handleLedOffsetBlur = (e: Event) => {
    const target = e.target as HTMLInputElement;
    const value = parseInt(target.value);
    if (!isNaN(value) && value >= 0 && value <= 1000) {
      setLedOffset(value);
      setLedOffsetInput(value.toString());
    } else {
      // Reset to current valid value
      setLedOffsetInput(ledOffset().toString());
      target.value = ledOffset().toString();
    }
  };

  const handleLedOffsetKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleLedOffsetBlur(e);
    }
  };

  // Sync input values with actual values
  createEffect(() => {
    setLedCountInput(ledCount().toString());
  });

  createEffect(() => {
    setLedOffsetInput(ledOffset().toString());
  });

  createEffect(() => {
    setAnimationSpeedInput(animationSpeed().toString());
  });

  // Load available boards on mount
  onMount(async () => {
    try {
      // 1. 首先停止所有正在运行的测试效果并重置模式
      console.log('🧹 Cleaning up any existing test effects...');
      try {
        // 禁用测试模式，这会停止所有测试效果并重置为AmbientLight模式
        await adaptiveApi.disableTestMode();
        console.log('✅ Test mode disabled and cleaned up');
      } catch (error) {
        console.warn('⚠️ Failed to disable test mode during cleanup:', error);
      }

      // 2. 加载可用的设备列表
      const boardList = await adaptiveApi.getBoards();
      setBoards(boardList);
      if (boardList.length > 0 && !selectedBoard()) {
        setSelectedBoard(boardList[0]);
      }
    } catch (error) {
      console.error('Failed to load boards:', error);
    }

    // Listen for board changes
    try {
      const unlisten = await adaptiveApi.onEvent<any>('BoardsChanged', (data) => {
        console.log('🔌 LED Strip Test - BoardsChanged event received:', data);

        // Extract boards from WebSocket message structure
        let boardList: BoardInfo[] = [];
        if (data && data.boards) {
          // WebSocket message format: { boards: [...] }
          boardList = Array.isArray(data.boards) ? data.boards : [];
          console.log('📋 Extracted boards from WebSocket message:', boardList);
        } else if (Array.isArray(data)) {
          // Direct array format
          boardList = data;
          console.log('📋 Using direct array format:', boardList);
        } else {
          console.warn('⚠️ Unexpected data format:', data);
        }

        console.log('✅ Setting board list:', boardList);
        setBoards(boardList);

        // If currently selected board is no longer available, select the first available one
        const currentBoard = selectedBoard();
        if (currentBoard) {
          const stillExists = boardList.find(board =>
            board.host === currentBoard.host &&
            board.address === currentBoard.address &&
            board.port === currentBoard.port
          );

          if (stillExists) {
            // Update to the new board object to reflect any status changes
            setSelectedBoard(stillExists);
          } else {
            // Current board is no longer available, select first available or null
            setSelectedBoard(boardList.length > 0 ? boardList[0] : null);
          }
        } else if (boardList.length > 0) {
          // No board was selected, select the first one
          setSelectedBoard(boardList[0]);
        }
      });

      // Cleanup listener when component is unmounted
      onCleanup(() => {
        unlisten();
      });
    } catch (error) {
      console.error('Failed to setup board change listener:', error);
    }
  });

  // Cleanup when component is unmounted
  onCleanup(() => {
    if (isRunning() && selectedBoard()) {
      // Use non-async cleanup to avoid the warning
      adaptiveApi.stopLedTestEffect({
        boardAddress: `${selectedBoard()!.address}:${selectedBoard()!.port}`,
        ledCount: ledCount(),
        ledType: ledType()
      }).then(() => {
        console.log('✅ Test effect stopped during cleanup');
      }).catch((error) => {
        console.error('Failed to stop test during cleanup:', error);
      });

      // Update local state immediately
      setIsRunning(false);
      setCurrentPattern(null);
    }
  });



  // Test patterns
  const testPatterns: TestPattern[] = [
    {
      name: t('ledTest.flowingRainbow'),
      description: t('ledTest.flowingRainbowDesc'),
      effect_type: 'FlowingRainbow'
    },
    {
      name: t('ledTest.groupCounting'),
      description: t('ledTest.groupCountingDesc'),
      effect_type: 'GroupCounting'
    },
    {
      name: t('ledTest.singleScan'),
      description: t('ledTest.singleScanDesc'),
      effect_type: 'SingleScan'
    },
    {
      name: t('ledTest.breathing'),
      description: t('ledTest.breathingDesc'),
      effect_type: 'Breathing'
    }
  ];



  // Test effect management - now handled by Rust backend

  const startTest = async (pattern: TestPattern) => {
    if (isRunning()) {
      await stopTest();
    }

    if (!selectedBoard()) {
      console.error('No board selected');
      return;
    }

    try {
      const effectConfig: TestEffectConfig = {
        effect_type: pattern.effect_type,
        led_count: ledCount(),
        led_type: ledType(),
        speed: 1.0 / (animationSpeed() / 50), // Convert animation speed to effect speed
        offset: ledOffset()
      };

      // 1. 首先启用测试模式
      console.log('🧪 启用测试模式...');
      await adaptiveApi.enableTestMode();

      // 2. 启动测试效果
      console.log('🚀 启动测试效果...');
      await adaptiveApi.startLedTestEffect({
        boardAddress: `${selectedBoard()!.address}:${selectedBoard()!.port}`,
        effectConfig: effectConfig,
        updateIntervalMs: animationSpeed()
      });

      setCurrentPattern(pattern);
      setIsRunning(true);
      console.log('✅ 测试效果启动成功');
    } catch (error) {
      console.error('Failed to start test effect:', error);
    }
  };

  const stopTest = async () => {
    const startTime = Date.now();
    console.log(`🛑 [${new Date().toISOString()}] stopTest函数被调用`);
    console.log('🔍 当前选中的板子:', selectedBoard());
    console.log('🔍 当前运行状态:', isRunning());
    console.log('🔍 当前测试模式:', currentPattern());

    if (!selectedBoard()) {
      console.log('⚠️ 没有选中的板子，直接更新UI状态');
      setIsRunning(false);
      setCurrentPattern(null);
      return;
    }

    // 立即更新UI状态，让用户感觉停止是即时的
    setIsRunning(false);
    setCurrentPattern(null);
    console.log(`🛑 [${new Date().toISOString()}] UI状态已更新，正在后台停止测试效果...`);

    // 后台异步停止测试效果，不阻塞UI
    const stopParams = {
      boardAddress: `${selectedBoard()!.address}:${selectedBoard()!.port}`,
      ledCount: ledCount(),
      ledType: ledType()
    };

    // 使用Promise.resolve().then()来确保异步执行，不阻塞UI
    Promise.resolve().then(async () => {
      try {
        // 1. 停止测试效果
        console.log(`🛑 [${new Date().toISOString()}] 停止测试效果...`);
        await adaptiveApi.stopLedTestEffect(stopParams);

        // 2. 禁用测试模式，恢复环境光模式
        console.log(`🌈 [${new Date().toISOString()}] 禁用测试模式，恢复环境光模式...`);
        await adaptiveApi.disableTestMode();

        const endTime = Date.now();
        const duration = endTime - startTime;
        console.log(`✅ [${new Date().toISOString()}] 测试效果已成功停止，已恢复环境光模式 (耗时: ${duration}ms)`);
      } catch (error) {
        console.error(`❌ [${new Date().toISOString()}] 停止测试效果失败:`, error);
        console.error('Error details:', error);

        // 如果停止失败，尝试强制禁用测试模式
        try {
          console.log(`🔄 [${new Date().toISOString()}] 尝试强制禁用测试模式...`);
          await adaptiveApi.disableTestMode();
          const endTime = Date.now();
          const duration = endTime - startTime;
          console.log(`✅ [${new Date().toISOString()}] 强制禁用测试模式成功 (总耗时: ${duration}ms)`);
        } catch (forceError) {
          const endTime = Date.now();
          const duration = endTime - startTime;
          console.error(`❌ [${new Date().toISOString()}] 强制禁用测试模式也失败了 (总耗时: ${duration}ms):`, forceError);
        }
      }
    });
  };

  // 测试LED配置数据发送
  const testLedConfigData = async () => {
    if (!selectedBoard()) {
      console.error('No board selected');
      return;
    }

    try {
      console.log('🚀 开始LED配置数据测试...');

      // 1. 启用测试模式
      await adaptiveApi.enableTestMode();
      console.log('✅ 测试模式已启用');

      // 2. 生成模拟LED配置数据
      const testData = generateLedConfigTestData();
      console.log(`📦 生成了 ${testData.length} 字节的测试数据`);

      // 3. 发送到选中的设备
      const boardAddress = `${selectedBoard()!.address}:${selectedBoard()!.port}`;

      console.log('🎯 发送测试数据到设备:', boardAddress);

      try {
        console.log(`📤 发送到 ${boardAddress}...`);

        await adaptiveApi.sendTestColorsToBoard({
          boardAddress: boardAddress,
          offset: 0,
          buffer: testData
        });

        console.log(`✅ 成功发送到 ${boardAddress}`);
      } catch (error) {
        console.error(`❌ 发送到 ${boardAddress} 失败:`, error);
      }

      console.log('🎉 LED配置数据测试完成');

    } catch (error) {
      console.error('❌ LED配置数据测试失败:', error);
    }
  };

  // 生成LED配置测试数据
  const generateLedConfigTestData = (): number[] => {
    // 模拟4个LED灯带的配置
    const strips = [
      { border: 'bottom', count: 38, ledType: 'SK6812', sequence: 1 },
      { border: 'right', count: 22, ledType: 'WS2812B', sequence: 2 },
      { border: 'top', count: 38, ledType: 'SK6812', sequence: 3 },
      { border: 'left', count: 22, ledType: 'WS2812B', sequence: 4 }
    ];

    // 生成边框测试颜色 - 最终解决所有角落颜色差异问题
    const borderColors: Record<string, Array<{r: number, g: number, b: number}>> = {
      'bottom': [{ r: 255, g: 0, b: 0 }, { r: 0, g: 255, b: 0 }],       // 红色 + 绿色
      'right': [{ r: 255, g: 255, b: 0 }, { r: 128, g: 0, b: 128 }],    // 黄色 + 紫色
      'top': [{ r: 255, g: 255, b: 255 }, { r: 0, g: 0, b: 0 }],        // 白色 + 黑色
      'left': [{ r: 255, g: 165, b: 0 }, { r: 0, g: 255, b: 255 }]      // 橙色 + 青色
    };

    const allColorBytes: number[] = [];

    // 按序列号排序
    strips.sort((a, b) => a.sequence - b.sequence);

    for (const strip of strips) {
      const colors = borderColors[strip.border];
      const halfCount = Math.floor(strip.count / 2);

      console.log(`生成 ${strip.border} 边框数据: ${strip.count} 个LED (${strip.ledType})`);

      // 前半部分使用第一种颜色
      for (let i = 0; i < halfCount; i++) {
        const color = colors[0];
        if (strip.ledType === 'SK6812') {
          allColorBytes.push(color.g, color.r, color.b, 0); // GRBW - 白色通道不点亮
        } else {
          allColorBytes.push(color.g, color.r, color.b); // GRB
        }
      }

      // 后半部分使用第二种颜色
      for (let i = halfCount; i < strip.count; i++) {
        const color = colors[1];
        if (strip.ledType === 'SK6812') {
          allColorBytes.push(color.g, color.r, color.b, 0); // GRBW - 白色通道不点亮
        } else {
          allColorBytes.push(color.g, color.r, color.b); // GRB
        }
      }
    }

    return allColorBytes;
  };



  return (
    <div class="container mx-auto p-6 space-y-6">
      {/* LED Preview */}
      <LedPreview class="mb-4" maxLeds={200} />

      <div class="card bg-base-200 shadow-xl">
        <div class="card-body">
          <h2 class="card-title text-2xl mb-4">{t('ledTest.title')}</h2>
          
          {/* Hardware Selection */}
          <div class="form-control w-full max-w-xs">
            <label class="label">
              <span class="label-text">{t('ledTest.selectHardwareBoard')}</span>
              <span class="label-text-alt">
                {boards().length > 0 ? `${boards().length} ${t('ledTest.devicesFound')}` : t('ledTest.searching')}
              </span>
            </label>
            <select
              class="select select-bordered w-full max-w-xs"
              value={selectedBoard()?.host || ''}
              onChange={(e) => {
                const board = boards().find(b => b.host === e.target.value);
                setSelectedBoard(board || null);
              }}
            >
              <option disabled value="">
                {boards().length > 0 ? t('ledTest.chooseBoard') : t('ledTest.noBoardsFound')}
              </option>
              <For each={boards()}>
                {(board) => {
                  const getStatusIcon = (status: BoardInfo['connect_status']) => {
                    if (status === 'Connected') return '🟢';
                    if (typeof status === 'object' && 'Connecting' in status) return '🟡';
                    return '🔴';
                  };

                  const getStatusText = (status: BoardInfo['connect_status']) => {
                    if (status === 'Connected') return t('ledTest.connected');
                    if (typeof status === 'object' && 'Connecting' in status) return t('ledTest.connecting');
                    return t('ledTest.disconnected');
                  };

                  return (
                    <option value={board.host}>
                      {getStatusIcon(board.connect_status)} {board.host} ({board.address}:{board.port}) - {getStatusText(board.connect_status)}
                    </option>
                  );
                }}
              </For>
            </select>
          </div>

          {/* LED Configuration */}
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mt-4">
            <div class="form-control">
              <label class="label">
                <span class="label-text">{t('ledTest.ledCount')}</span>
              </label>
              <input
                type="number"
                class="input input-bordered w-full text-center text-lg"
                value={ledCountInput()}
                min="1"
                max="1000"
                onInput={handleLedCountInput}
                onBlur={handleLedCountBlur}
                onKeyDown={handleLedCountKeyDown}
              />
            </div>

            <div class="form-control">
              <label class="label">
                <span class="label-text">{t('ledTest.ledType')}</span>
              </label>
              <select
                class="select select-bordered w-full"
                value={ledType()}
                onChange={(e) => setLedType(e.target.value as 'WS2812B' | 'SK6812')}
              >
                <option value="WS2812B">WS2812B</option>
                <option value="SK6812">SK6812</option>
              </select>
            </div>

            <div class="form-control">
              <label class="label">
                <span class="label-text">{t('ledTest.ledOffset')}</span>
              </label>
              <input
                type="number"
                class="input input-bordered w-full text-center text-lg"
                value={ledOffsetInput()}
                min="0"
                max="1000"
                onInput={handleLedOffsetInput}
                onBlur={handleLedOffsetBlur}
                onKeyDown={handleLedOffsetKeyDown}
              />
            </div>

            <div class="form-control">
              <label class="label">
                <span class="label-text">{t('ledTest.animationSpeed')}</span>
              </label>
              <input
                type="number"
                class="input input-bordered w-full text-center"
                value={animationSpeedInput()}
                min="16"
                max="3600000"
                step="1"
                onInput={handleAnimationSpeedInput}
                onBlur={handleAnimationSpeedBlur}
                onKeyDown={handleAnimationSpeedKeyDown}
              />
            </div>
          </div>
        </div>
      </div>

      {/* Test Patterns */}
      <div class="card bg-base-200 shadow-xl">
        <div class="card-body">
          <h3 class="card-title text-xl mb-4">Test Patterns</h3>
          
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <For each={testPatterns}>
              {(pattern) => (
                <div class="card bg-base-100 shadow-md">
                  <div class="card-body">
                    <h4 class="card-title text-lg">{pattern.name}</h4>
                    <p class="text-sm opacity-70 mb-4">{pattern.description}</p>
                    
                    <div class="card-actions justify-end">
                      <Show 
                        when={currentPattern() === pattern && isRunning()}
                        fallback={
                          <button
                            class="btn btn-primary"
                            onClick={() => startTest(pattern)}
                            disabled={!selectedBoard()}
                          >
                            {t('ledTest.startTestButton')}
                          </button>
                        }
                      >
                        <button
                          class="btn btn-error"
                          onClick={() => stopTest()}
                        >
                          {t('ledTest.stopTest')}
                        </button>
                      </Show>
                    </div>
                  </div>
                </div>
              )}
            </For>
          </div>

          {/* LED配置数据测试按钮 */}
          <div class="divider">LED配置数据测试</div>
          <div class="flex gap-4">
            <button
              class="btn btn-secondary"
              onClick={testLedConfigData}
              disabled={!selectedBoard()}
            >
              🔧 测试LED配置数据发送
            </button>
          </div>

          <Show when={isRunning()}>
            <div class="alert alert-info mt-4">
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-6 h-6">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
              <span>Test pattern "{currentPattern()?.name}" is running on {selectedBoard()?.host}</span>
            </div>
          </Show>
          
          <Show when={!selectedBoard()}>
            <div class="alert alert-warning mt-4">
              <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.728-.833-2.498 0L3.732 16c-.77.833.192 2.5 1.732 2.5z" />
              </svg>
              <span>Please select a hardware board to start testing</span>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
};
