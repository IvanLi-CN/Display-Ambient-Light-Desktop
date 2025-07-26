export type Language = 'zh-CN' | 'en-US';

export interface TranslationDict {
  // Navigation
  nav: {
    title: string;
    info: string;
    displays: string;
    ledConfiguration: string;
    colorCalibration: string;
    ledTest: string;
    settings: string;
  };
  
  // Common UI elements
  common: {
    version: string;
    primary: string;
    save: string;
    cancel: string;
    reset: string;
    close: string;
    fullscreen: string;
    exitFullscreen: string;
    loading: string;
    error: string;
    success: string;
    warning: string;
    confirm: string;
    delete: string;
    edit: string;
    add: string;
    remove: string;
    enable: string;
    disable: string;
    start: string;
    stop: string;
    test: string;
    apply: string;
    refresh: string;
    realtime: string;
    back: string;
    clear: string;
    confirmUnsavedChanges: string;
  };
  
  // Info page
  info: {
    title: string;
    boardInfo: string;
    systemInfo: string;
    deviceName: string;
    ipAddress: string;
    macAddress: string;
    firmwareVersion: string;
    hardwareVersion: string;
    uptime: string;
    status: string;
    connected: string;
    disconnected: string;
    lastSeen: string;
    port: string;
    latency: string;
    hostname: string;
    deviceCount: string;
    noDevicesFound: string;
    checkConnection: string;
    // Device status
    timeout: string;
    connecting: string;
    unknown: string;
  };
  
  // Display page
  displays: {
    title: string;
    count: string;
    noDisplays: string;
    checkConnection: string;
    displayInfo: string;
    resolution: string;
    refreshRate: string;
    colorDepth: string;
    isPrimary: string;
    position: string;
    size: string;
    scaleFactor: string;
    lastModified: string;
    displayCount: string;
    noDisplaysFound: string;
    brightnessSettings: string;
    currentBrightness: string;
    maxBrightness: string;
    minBrightness: string;
    contrastSettings: string;
    currentContrast: string;
    maxContrast: string;
    minContrast: string;
    modeSettings: string;
    currentMode: string;
    maxMode: string;
    minMode: string;
    // Display info panel specific
    id: string;
    scale: string;
  };
  
  // LED Strip Configuration
  ledConfig: {
    title: string;
    displaySelection: string;
    ledStripConfig: string;
    ledCount: string;
    ledType: string;
    border: string;
    count: string;
    position: string;
    top: string;
    bottom: string;
    left: string;
    right: string;
    preview: string;
    configuration: string;
    sorter: string;
    moveUp: string;
    moveDown: string;
    reverse: string;
    rgb: string;
    rgbw: string;
    segments: string;
    totalLeds: string;
    saveConfig: string;
    loadConfig: string;
    stripSorting: string;
    realtimePreview: string;
    sortingTip: string;
    displayConfiguration: string;
    visualEditor: string;
    displayTip: string;
    ledCountControl: string;
    realtimeAdjustment: string;
    decreaseLedCount: string;
    increaseLedCount: string;
    display: string;
    driver: string;
    sequence: string;
    startOffset: string;
    endOffset: string;
    testStrip: string;
    configPanel: string;
    controlTip: string;
  };

  // Single Display LED Strip Configuration
  singleDisplayConfig: {
    title: string;
    displayNotFound: string;
    displayVisualization: string;
    displayInfo: string;
    ledConfiguration: string;
    selectStripToConfig: string;
    selectOrCreateStrip: string;
    clearConfig: string;
    confirmClear: string;
    confirmDeleteStrip: string;
    virtualDisplay: string;
    virtualDisplayDesc: string;
    virtualDisplayPlaceholder: string;
    virtualDisplayInstructions: string;
    colorIndicator: string;
    colorIndicatorDesc: string;
    stripConfig: string;
    dataDirection: string;
    normal: string;
    reversed: string;
    driverSelection: string;
    driver: string;
    stripOrder: string;
    positionOffset: string;
    startOffset: string;
    endOffset: string;
    // Save status
    configSaved: string;
    saveFailed: string;
    saving: string;
    saveConfig: string;
  };
  
  // Color Calibration
  colorCalibration: {
    title: string;
    colorCalibration: string;
    redChannel: string;
    greenChannel: string;
    blueChannel: string;
    whiteChannel: string;
    brightness: string;
    temperature: string;
    resetToDefault: string;
    fullscreenMode: string;
    normalMode: string;
    instructions: string;
    helpText: string;
    compareColors: string;
    adjustValues: string;
    dragToMove: string;
    back: string;
    colorTest: string;
    clickToTest: string;
    colorTestTip: string;
    rgbAdjustment: string;
    realtimeAdjustment: string;
    usageInstructions: string;
    recommendedMethod: string;
    adjustmentTips: string;
    comparisonMethod: string;
    fullscreenTip: string;
    dragTip: string;
    redStrong: string;
    greenStrong: string;
    blueStrong: string;
    whiteYellow: string;
    whiteBlue: string;
    whiteComparison: string;
    colorComparison: string;
    environmentTest: string;
    resetNote: string;
    fullscreenComparisonTip: string;
    draggable: string;
    exitFullscreen: string;
    notEnabled: string;
    // Missing white balance instructions
    dragPanelTip: string;
    compareColorsTip: string;
  };
  
  // LED Test
  ledTest: {
    title: string;
    testEffects: string;
    staticColor: string;
    rainbow: string;
    breathing: string;
    wave: string;
    chase: string;
    twinkle: string;
    fire: string;
    speed: string;
    brightness: string;
    color: string;
    startTest: string;
    stopTest: string;
    testRunning: string;
    testStopped: string;
    selectEffect: string;
    effectSettings: string;
    flowingRainbow: string;
    flowingRainbowDesc: string;
    groupCounting: string;
    groupCountingDesc: string;
    singleScan: string;
    singleScanDesc: string;
    breathingDesc: string;
    // LED test form labels
    ledCount: string;
    ledType: string;
    ledOffset: string;
    animationSpeed: string;
    startTestButton: string;
    // Hardware selection
    selectHardwareBoard: string;
    devicesFound: string;
    searching: string;
    chooseBoard: string;
    noBoardsFound: string;
    connected: string;
    connecting: string;
    disconnected: string;
  };
  
  // Error messages
  errors: {
    failedToLoad: string;
    failedToSave: string;
    failedToConnect: string;
    invalidConfiguration: string;
    deviceNotFound: string;
    networkError: string;
    unknownError: string;
  };

  // Settings page
  settings: {
    title: string;
    language: string;
    languageSelection: string;
    languageDescription: string;
    theme: string;
    themeSelection: string;
    themeDescription: string;
    themeLight: string;
    themeDark: string;
    themeAuto: string;
    themeSuccess: string;
    themeError: string;
    lightThemes: string;
    darkThemes: string;
    nightModeTheme: string;
    nightModeThemeDescription: string;
    enableNightModeTheme: string;
    enableNightModeThemeDescription: string;
    autoStart: string;
    autoStartDescription: string;
    autoStartEnabled: string;
    autoStartDisabled: string;
    autoStartSuccess: string;
    autoStartError: string;
    general: string;
    system: string;
    about: string;
    appearance: string;
  };

  about: {
    title: string;
    version: string;
    author: string;
    description: string;
    repository: string;
    license: string;
    homepage: string;
    close: string;
    openRepository: string;
    openHomepage: string;
  };

  // System tray menu
  tray: {
    ambientLight: string;
    ambientLightEnabled: string;
    ambientLightDisabled: string;
    ledPreview: string;
    ledPreviewEnabled: string;
    ledPreviewDisabled: string;
    info: string;
    ledConfiguration: string;
    colorCalibration: string;
    ledTest: string;
    settings: string;
    autoStart: string;
    quit: string;
    show: string;
    hide: string;
  };

  // Ambient light control
  ambientLight: {
    title: string;
    enabled: string;
    disabled: string;
    statusEnabled: string;
    statusDisabled: string;
    description: string;
    descriptionEnabled: string;
    descriptionDisabled: string;
    toggleFailed: string;
  };

  // LED Status
  ledStatus: {
    title: string;
    mode: string;
    frequency: string;
    data: string;
    led: string;
    update: string;
    received: string;
    connected: string;
    disconnected: string;
    waitingForData: string;
    websocketDisconnected: string;
    testMode: string;
    modes: {
      None: string;
      AmbientLight: string;
      StripConfig: string;
      TestEffect: string;
      ColorCalibration: string;
    };
  };
}
