# Device Auto-Refresh Testing Guide

## Test Scenarios

### 1. Initial Load Test
**Expected Behavior**: Device list loads automatically when component mounts

**Steps**:
1. Start the application: `npm run tauri dev`
2. Navigate to LED Strip Test page
3. Observe the device dropdown

**Expected Results**:
- Device dropdown shows "Searching..." if no devices found
- Device dropdown shows device count if devices are found
- First available device is automatically selected
- Status icons appear next to device names

### 2. Device Discovery Test
**Expected Behavior**: New devices appear automatically when discovered

**Steps**:
1. Start with no devices connected
2. Connect a device to the network
3. Wait for device discovery (should be automatic)

**Expected Results**:
- Device count updates automatically
- New device appears in dropdown
- If no device was selected, new device gets selected automatically
- Status icon shows connection state

### 3. Device Disconnection Test
**Expected Behavior**: Disconnected devices are handled gracefully

**Steps**:
1. Start with connected devices
2. Select a device in the dropdown
3. Disconnect the selected device from network
4. Wait for connection timeout

**Expected Results**:
- Device status changes to disconnected (🔴)
- If device becomes unavailable, another device is selected automatically
- Device count updates
- UI remains responsive

### 4. Connection Status Test
**Expected Behavior**: Status indicators reflect actual device states

**Steps**:
1. Observe devices in different connection states
2. Check status icons and text

**Expected Results**:
- 🟢 "Connected" for responsive devices
- 🟡 "Connecting" for devices in retry state
- 🔴 "Disconnected" for unresponsive devices
- Status text matches icon state

### 5. UI Responsiveness Test
**Expected Behavior**: Interface remains responsive during device changes

**Steps**:
1. Rapidly connect/disconnect devices
2. Interact with other UI elements during device changes
3. Switch between pages and return

**Expected Results**:
- No UI freezing or lag
- Event listeners are properly cleaned up
- No memory leaks
- Smooth transitions

## Verification Checklist

- [ ] Device dropdown shows correct device count
- [ ] Status icons display correctly (🟢🟡🔴)
- [ ] Automatic device selection works
- [ ] Event listeners are cleaned up on component unmount
- [ ] No TypeScript errors in console
- [ ] No runtime errors in console
- [ ] Performance remains good with multiple devices
- [ ] UI updates smoothly without flickering

## Common Issues to Watch For

1. **Memory Leaks**: Event listeners not cleaned up
2. **Type Errors**: Incorrect BoardInfo type handling
3. **Selection Logic**: Device selection not updating correctly
4. **Performance**: UI lag during rapid device changes
5. **State Consistency**: UI state not matching actual device state

## Debug Information

Check browser console for:
- `boards_changed` events
- Device list updates
- Selection changes
- Any error messages

Check Tauri logs for:
- Device discovery messages
- Connection status changes
- mDNS service events
