import { ColorSlider } from './color-slider';
import { TestColorsBg } from './test-colors-bg';

export const WhiteBalance = () => {
  const exit = () => {
    window.history.back();
  };

  return (
    <section class="select-none">
      <div class="absolute top-0 left-0 right-0 bottom-0">
        <TestColorsBg />
      </div>
      <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-10/12 max-w-lg bg-stone-200 p-5 rounded-xl drop-shadow">
        <label class="flex items-center gap-2">
          <span class="w-3 block">R:</span>
          <ColorSlider class="from-cyan-500 to-red-500" />
        </label>
        <label class="flex items-center gap-2">
          <span class="w-3 block">G:</span>
          <ColorSlider class="from-pink-500 to-green-500" />
        </label>
        <label class="flex items-center gap-2">
          <span class="w-3 block">B:</span>
          <ColorSlider class="from-yellow-500 to-blue-500" />
        </label>
        <label class="flex items-center gap-2">
          <span class="w-3 block">W:</span>
          <ColorSlider class="from-yellow-50 to-cyan-50" />
        </label>
        <button
          class="absolute -right-4 -top-4 rounded-full aspect-square bg-stone-300 p-1 shadow border border-stone-400"
          onClick={exit}
        >
          X
        </button>
        <button class="absolute -right-4 -bottom-4 rounded-full aspect-square bg-stone-300 p-1 shadow border border-stone-400">
          R
        </button>
      </div>
    </section>
  );
};
