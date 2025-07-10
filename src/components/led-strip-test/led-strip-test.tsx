import { createSignal, createEffect, For, Show, onCleanup } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useLanguage } from '../../i18n/index';

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
  const { t } = useLanguage();
  const [boards, setBoards] = createSignal<BoardInfo[]>([]);
  const [selectedBoard, setSelectedBoard] = createSignal<BoardInfo | null>(null);
  const [ledCount, setLedCount] = createSignal(60);
  const [ledType, setLedType] = createSignal<'WS2812B' | 'SK6812'>('WS2812B');
  const [isRunning, setIsRunning] = createSignal(false);
  const [currentPattern, setCurrentPattern] = createSignal<TestPattern | null>(null);
  const [animationSpeed, setAnimationSpeed] = createSignal(33); // ~30fps

  // Temporary input values for better UX
  const [ledCountInput, setLedCountInput] = createSignal('60');
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

  // Sync input values with actual values
  createEffect(() => {
    setLedCountInput(ledCount().toString());
  });

  createEffect(() => {
    setAnimationSpeedInput(animationSpeed().toString());
  });

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
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
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
