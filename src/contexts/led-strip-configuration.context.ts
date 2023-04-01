import { createContext } from 'solid-js';
import { Borders } from '../constants/border';

export type LedStripConfigurationContextType = [
  {
    selectedStripPart: {
      displayId: number;
      border: Borders;
    } | null;
  },
  {
    setSelectedStripPart: (v: { displayId: number; border: Borders } | null) => void;
  },
];

export const LedStripConfigurationContext =
  createContext<LedStripConfigurationContextType>([
    {
      selectedStripPart: null,
    },
    {
      setSelectedStripPart: () => {},
    },
  ]);
