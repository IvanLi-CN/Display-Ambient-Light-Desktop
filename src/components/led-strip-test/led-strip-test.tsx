import { createSignal, createEffect, For, Show, onCleanup } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

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
}

export const LedStripTest = () => {
  const [boards, setBoards] = createSignal<BoardInfo[]>([]);
  const [selectedBoard, setSelectedBoard] = createSignal<BoardInfo | null>(null);
  const [ledCount, setLedCount] = createSignal(60);
  const [ledType, setLedType] = createSignal<'RGB' | 'RGBW'>('RGB');
  const [isRunning, setIsRunning] = createSignal(false);
  const [currentPattern, setCurrentPattern] = createSignal<TestPattern | null>(null);
  const [animationSpeed, setAnimationSpeed] = createSignal(33); // ~30fps

  // Load available boards and listen for changes
  createEffect(() => {
    // Initial load
    invoke<BoardInfo[]>('get_boards').then((boardList) => {
      setBoards(boardList);
      if (boardList.length > 0 && !selectedBoard()) {
        setSelectedBoard(boardList[0]);
      }
    }).catch((error) => {
      console.error('Failed to load boards:', error);
    });

    // Listen for board changes
    const unlisten = listen<BoardInfo[]>('boards_changed', (event) => {
      const boardList = event.payload;
      setBoards(boardList);

      // If currently selected board is no longer available, select the first available one
      const currentBoard = selectedBoard();
      if (currentBoard) {
        const stillExists = boardList.find(board =>
          board.host === currentBoard.host &&
          board.address === currentBoard.address &&
          board.port === currentBoard.port
        );

        if (!stillExists) {
          // Current board is no longer available, select first available or null
          setSelectedBoard(boardList.length > 0 ? boardList[0] : null);
        }
      } else if (boardList.length > 0) {
        // No board was selected, select the first one
        setSelectedBoard(boardList[0]);
      }
    });

    // Cleanup listener when effect is disposed
    onCleanup(() => {
      unlisten.then((unlistenFn) => unlistenFn());
    });
  });

  // Cleanup when component is unmounted
  onCleanup(() => {
    if (isRunning() && selectedBoard()) {
      // Stop the test effect in backend
      invoke('stop_led_test_effect', {
        boardAddress: `${selectedBoard()!.address}:${selectedBoard()!.port}`,
        ledCount: ledCount(),
        ledType: ledType()
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
      name: '流光效果',
      description: '彩虹色流光，用于测试灯带方向',
      effect_type: 'FlowingRainbow'
    },
    {
      name: '十个一组计数',
      description: '每十个LED一组不同颜色，用于快速计算灯珠数量',
      effect_type: 'GroupCounting'
    },
    {
      name: '单色扫描',
      description: '单个LED依次点亮，用于精确测试每个LED位置',
      effect_type: 'SingleScan'
    },
    {
      name: '呼吸灯',
      description: '整条灯带呼吸效果，用于测试整体亮度',
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
        speed: 1.0 / (animationSpeed() / 50) // Convert animation speed to effect speed
      };

      // Start the test effect in Rust backend
      await invoke('start_led_test_effect', {
        boardAddress: `${selectedBoard()!.address}:${selectedBoard()!.port}`,
        effectConfig: effectConfig,
        updateIntervalMs: animationSpeed()
      });

      setCurrentPattern(pattern);
      setIsRunning(true);
    } catch (error) {
      console.error('Failed to start test effect:', error);
    }
  };

  const stopTest = async () => {
    if (!selectedBoard()) {
      setIsRunning(false);
      setCurrentPattern(null);
      return;
    }

    try {
      // Stop the test effect in Rust backend
      await invoke('stop_led_test_effect', {
        boardAddress: `${selectedBoard()!.address}:${selectedBoard()!.port}`,
        ledCount: ledCount(),
        ledType: ledType()
      });

      // Only update UI state after successful backend call
      setIsRunning(false);
      setCurrentPattern(null);
    } catch (error) {
      console.error('Failed to stop test effect:', error);
      // Still update UI state even if backend call fails
      setIsRunning(false);
      setCurrentPattern(null);
    }
  };



  return (
    <div class="container mx-auto p-6 space-y-6">
      <div class="card bg-base-200 shadow-xl">
        <div class="card-body">
          <h2 class="card-title text-2xl mb-4">LED Strip Testing</h2>
          
          {/* Hardware Selection */}
          <div class="form-control w-full max-w-xs">
            <label class="label">
              <span class="label-text">Select Hardware Board</span>
              <span class="label-text-alt">
                {boards().length > 0 ? `${boards().length} device(s) found` : 'Searching...'}
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
                {boards().length > 0 ? 'Choose a board' : 'No boards found'}
              </option>
              <For each={boards()}>
                {(board) => {
                  const getStatusIcon = (status: BoardInfo['connect_status']) => {
                    if (status === 'Connected') return '🟢';
                    if (typeof status === 'object' && 'Connecting' in status) return '🟡';
                    return '🔴';
                  };

                  const getStatusText = (status: BoardInfo['connect_status']) => {
                    if (status === 'Connected') return 'Connected';
                    if (typeof status === 'object' && 'Connecting' in status) return 'Connecting';
                    return 'Disconnected';
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
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
            <div class="form-control">
              <label class="label">
                <span class="label-text">LED Count</span>
              </label>
              <input 
                type="number" 
                class="input input-bordered w-full text-center text-lg"
                value={ledCount()}
                min="1"
                max="1000"
                onInput={(e) => setLedCount(parseInt(e.target.value) || 60)}
              />
            </div>
            
            <div class="form-control">
              <label class="label">
                <span class="label-text">LED Type</span>
              </label>
              <select 
                class="select select-bordered w-full"
                value={ledType()}
                onChange={(e) => setLedType(e.target.value as 'RGB' | 'RGBW')}
              >
                <option value="RGB">RGB</option>
                <option value="RGBW">RGBW</option>
              </select>
            </div>
            
            <div class="form-control">
              <label class="label">
                <span class="label-text">Animation Speed (ms)</span>
              </label>
              <input
                type="number"
                class="input input-bordered w-full text-center"
                value={animationSpeed()}
                min="16"
                max="200"
                step="1"
                onInput={(e) => setAnimationSpeed(parseInt(e.target.value) || 33)}
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
                            Start Test
                          </button>
                        }
                      >
                        <button
                          class="btn btn-error"
                          onClick={() => stopTest()}
                        >
                          Stop Test
                        </button>
                      </Show>
                    </div>
                  </div>
                </div>
              )}
            </For>
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
