[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $Executable,
    [ValidateRange(1, 1000)] [int] $Samples = 20,
    [switch] $TestClipboard,
    [switch] $UseFunctionKeys
)

$ErrorActionPreference = 'Stop'
if (Get-Process -Name qapture -ErrorAction SilentlyContinue) { throw 'Quit Qapture before running this test.' }
if ([Threading.Thread]::CurrentThread.ApartmentState -ne 'STA') { throw 'Run with powershell.exe -STA.' }

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class QaptureProbe {
    [StructLayout(LayoutKind.Sequential)]
    public struct Point { public int X, Y; }
    [StructLayout(LayoutKind.Sequential)]
    public struct Bounds { public int Left, Top, Right, Bottom; }

    [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr window);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string className, string title);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr window, out Bounds bounds);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr window, uint message, UIntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool GetPhysicalCursorPos(out Point point);
    [DllImport("user32.dll")] public static extern bool SetPhysicalCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr info);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint x, uint y, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern bool RegisterHotKey(IntPtr window, int id, uint modifiers, uint key);
    [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr window);
    [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr window, IntPtr dc);
    [DllImport("gdi32.dll")] public static extern uint GetPixel(IntPtr dc, int x, int y);
    [DllImport("dwmapi.dll")] public static extern int DwmFlush();
    [DllImport("user32.dll")] public static extern int CountClipboardFormats();
    [DllImport("user32.dll")] public static extern bool OpenClipboard(IntPtr window);
    [DllImport("user32.dll")] public static extern bool CloseClipboard();
    [DllImport("user32.dll")] public static extern IntPtr GetClipboardData(uint format);
    [DllImport("kernel32.dll")] public static extern IntPtr GlobalLock(IntPtr handle);
    [DllImport("kernel32.dll")] public static extern bool GlobalUnlock(IntPtr handle);

    public static void Key(byte key, bool down) { keybd_event(key, 0, down ? 0u : 2u, UIntPtr.Zero); }
    public static void Tap(byte key) { Key(key, true); Key(key, false); }
    public static void Shortcut(byte key, bool functionKeys) {
        if (functionKeys) {
            Key(0x11, true); Key(0x10, true); Tap(key); Key(0x10, false); Key(0x11, false);
        } else {
            Key(0x12, true); Tap(key); Key(0x12, false);
        }
    }
    public static uint ScreenColor(int x, int y) {
        var dc = GetDC(IntPtr.Zero);
        try {
            var color = GetPixel(dc, x, y);
            if (color == UInt32.MaxValue) throw new InvalidOperationException("Fixture point is off screen.");
            return ((color & 255u) << 16) | (color & 0xff00u) | ((color >> 16) & 255u);
        } finally { ReleaseDC(IntPtr.Zero, dc); }
    }
    public static int ClipboardPixel(int width, int height, int x, int y) {
        if (!OpenClipboard(IntPtr.Zero)) return -1;
        try {
            var handle = GetClipboardData(8);
            if (handle == IntPtr.Zero) return -1;
            var data = GlobalLock(handle);
            if (data == IntPtr.Zero) return -1;
            try {
                if (Marshal.ReadInt32(data, 4) != width || Marshal.ReadInt32(data, 8) != height
                    || Marshal.ReadInt16(data, 14) != 32)
                    throw new InvalidOperationException("Unexpected native clipboard bitmap dimensions or format.");
                return Marshal.ReadInt32(data, 40 + 4 * ((height - 1 - y) * width + x)) & 0xffffff;
            } finally { GlobalUnlock(handle); }
        } finally { CloseClipboard(); }
    }
}
'@

function Assert([bool] $Condition, [string] $Message) {
    if (!$Condition) { throw $Message }
}

function Wait-For([scriptblock] $Condition, [string] $Description, [int] $TimeoutMs = 5000) {
    $watch = [Diagnostics.Stopwatch]::StartNew()
    do {
        [Windows.Forms.Application]::DoEvents()
        $result = & $Condition
        if ($result) { return $result }
        Start-Sleep -Milliseconds 12
    } while ($watch.ElapsedMilliseconds -lt $TimeoutMs)
    throw "Timed out waiting for $Description."
}

function Read-Entries {
    if (Test-Path -LiteralPath $log) {
        Get-Content -LiteralPath $log | ForEach-Object { if ($_) { ConvertFrom-Json $_ } }
    }
}

function Wait-Reports([int] $Count) {
    Wait-For { @(Read-Entries | Where-Object event -eq 'capture-finished').Count -ge $Count } 'capture completion' | Out-Null
}

function Open-Overlay([switch] $VisibleOnly) {
    [QaptureProbe]::Shortcut($regionKey, $UseFunctionKeys)
    Wait-For {
        $window = [QaptureProbe]::FindWindow('QaptureSelectionOverlay', 'Qapture region selection')
        if ($window -ne [IntPtr]::Zero -and [QaptureProbe]::IsWindowVisible($window) -and
            ($VisibleOnly -or [QaptureProbe]::GetForegroundWindow() -eq $window)) { $window }
    } 'the region overlay'
}

function Start-RegionDrag([int] $X, [int] $Y, [int] $Width, [int] $Height) {
    [QaptureProbe]::SetPhysicalCursorPos($X, $Y) | Out-Null
    [QaptureProbe]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 50
    [QaptureProbe]::SetPhysicalCursorPos(($X + $Width), ($Y + $Height)) | Out-Null
    Start-Sleep -Milliseconds 50
}

function Complete-RegionDrag {
    [QaptureProbe]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero)
    Wait-For {
        $window = [QaptureProbe]::FindWindow([NullString]::Value, "Qapture $([char]0x2014) Quick editor")
        if ($window -ne [IntPtr]::Zero -and [QaptureProbe]::IsWindowVisible($window)) { $window }
    } 'the quick editor after mouse release'
}

function Get-ProcessTree([int] $Root) {
    $processes = @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId)
    $ids = [Collections.Generic.HashSet[int]]::new()
    $ids.Add($Root) | Out-Null
    do {
        $count = $ids.Count
        foreach ($item in $processes) {
            if ($ids.Contains([int]$item.ParentProcessId)) { $ids.Add([int]$item.ProcessId) | Out-Null }
        }
    } while ($ids.Count -ne $count)
    Get-Process -Id @($ids) -ErrorAction SilentlyContinue
}

$exe = (Resolve-Path -LiteralPath $Executable).Path
$regionKey = if ($UseFunctionKeys) { 0x78 } else { 0x41 }
$fullKey = if ($UseFunctionKeys) { 0x79 } else { 0x53 }
$modifiers = if ($UseFunctionKeys) { 6 } else { 1 }
$previousDiagnostics = $env:QAPTURE_DIAGNOSTICS
$log = Join-Path ([IO.Path]::GetTempPath()) "qapture-desktop-$PID.jsonl"
$env:QAPTURE_DIAGNOSTICS = $log
$previousDpi = [QaptureProbe]::SetThreadDpiAwarenessContext([IntPtr](-4))
$previousWindow = [QaptureProbe]::GetForegroundWindow()
$previousPointer = New-Object QaptureProbe+Point
[QaptureProbe]::GetPhysicalCursorPos([ref]$previousPointer) | Out-Null
$windows = @()
$app = $second = $dropdown = $null

try {
    if ($TestClipboard -and [QaptureProbe]::CountClipboardFormats() -ne 0) {
        throw 'Clipboard data is present. The positive clipboard test requires an empty test clipboard.'
    }
    $startup = [Diagnostics.Stopwatch]::StartNew()
    $app = Start-Process -FilePath $exe -PassThru
    Wait-For { Read-Entries | Where-Object event -eq 'dashboard-ready' } 'dashboard rendering' 12000 | Out-Null
    $startup.Stop()
    $main = Wait-For { $app.Refresh(); if ($app.MainWindowHandle -ne [IntPtr]::Zero) { $app.MainWindowHandle } } 'dashboard window'
    [QaptureProbe]::PostMessage($main, 0x10, [UIntPtr]::Zero, [IntPtr]::Zero) | Out-Null
    Wait-For { ![QaptureProbe]::IsWindowVisible($main) } 'close-to-tray' | Out-Null

    $idleStart = @(Get-ProcessTree $app.Id)
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $cpuStart = @{}
    foreach ($process in $idleStart) { $cpuStart[$process.Id] = $process.CPU }
    Start-Sleep -Seconds 4
    $idleEnd = @(Get-ProcessTree $app.Id)
    $cpuDelta = 0.0
    foreach ($process in $idleEnd) {
        if ($cpuStart.ContainsKey($process.Id)) { $cpuDelta += [Math]::Max(0.0, $process.CPU - $cpuStart[$process.Id]) }
    }
    $elapsed = $timer.Elapsed.TotalSeconds
    $workingSet = ($idleEnd | Measure-Object WorkingSet64 -Sum).Sum
    'Idle tray: {0:N1} MiB combined working set, {1:N2}% host CPU over {2:N1} s, {3} processes' -f ($workingSet / 1MB), (100 * $cpuDelta / $elapsed / [Environment]::ProcessorCount), $elapsed, $idleEnd.Count

    foreach ($screen in [Windows.Forms.Screen]::AllScreens) {
        $form = New-Object Windows.Forms.Form
        $form.FormBorderStyle = 'None'
        $form.StartPosition = 'Manual'
        $form.Bounds = $screen.Bounds
        $form.Text = 'Qapture test fixture'
        $form.BackColor = [Drawing.Color]::FromArgb(52, 81, 145)
        $form.Show()
        $windows += $form
        "Display fixture: $($form.Bounds) at $([QaptureProbe]::GetDpiForWindow($form.Handle)) DPI"
        if ($screen.Primary) { $fixture = $form }
    }
    [QaptureProbe]::SetForegroundWindow($fixture.Handle) | Out-Null
    $dropdown = New-Object Windows.Forms.ContextMenuStrip
    $dropdown.Items.Add('Qapture fixture menu') | Out-Null
    $dropdown.Items.Add('Second item') | Out-Null
    $dropdown.Show($fixture, [Drawing.Point]::new(250, 180))
    [Windows.Forms.Application]::DoEvents()
    $menuX = $dropdown.Bounds.Left + 8
    $menuY = $dropdown.Bounds.Top + 8
    $menuColor = Wait-For {
        $color = [QaptureProbe]::ScreenColor($menuX, $menuY)
        if ($color -ne 0x345191) { $color }
    } 'the fixture dropdown'
    $overlay = Open-Overlay -VisibleOnly
    $bounds = New-Object QaptureProbe+Bounds
    [QaptureProbe]::GetWindowRect($overlay, [ref]$bounds) | Out-Null
    $left = ($windows | ForEach-Object { $_.Left } | Measure-Object -Minimum).Minimum
    $top = ($windows | ForEach-Object { $_.Top } | Measure-Object -Minimum).Minimum
    $right = ($windows | ForEach-Object { $_.Right } | Measure-Object -Maximum).Maximum
    $bottom = ($windows | ForEach-Object { $_.Bottom } | Measure-Object -Maximum).Maximum
    Assert ($bounds.Left -eq $left -and $bounds.Top -eq $top -and $bounds.Right -eq $right -and $bounds.Bottom -eq $bottom) 'Overlay bounds differed from the physical monitor union.'
    [QaptureProbe]::Shortcut($regionKey, $UseFunctionKeys)
    [QaptureProbe]::Shortcut($fullKey, $UseFunctionKeys)
    Start-Sleep -Milliseconds 100
    Assert ([QaptureProbe]::FindWindow('QaptureSelectionOverlay', 'Qapture region selection') -eq $overlay) 'Repeated hotkeys replaced the active overlay.'

    $regionX = $fixture.Left + 20
    $regionY = $fixture.Top + 100
    $regionWidth = [Math]::Min(1920, $fixture.Width - 40)
    $regionHeight = [Math]::Min(1080, $fixture.Height - 120)
    # Close the source popup after the snapshot so the observed pixel can only be from the overlay.
    $dropdown.Close()
    [Windows.Forms.Application]::DoEvents()
    Start-RegionDrag $regionX $regionY $regionWidth $regionHeight
    if ($UseFunctionKeys) {
        Wait-For { [QaptureProbe]::ScreenColor($menuX, $menuY) -eq $menuColor } 'the frozen dropdown pixel' | Out-Null
    }
    Wait-For { [QaptureProbe]::ScreenColor(($regionX + 9), ($regionY - 15)) -eq 0x131e36 } 'the dimmed pixels outside the selection' | Out-Null
    Wait-For { [QaptureProbe]::GetForegroundWindow() -eq $overlay } 'selection keyboard focus' | Out-Null
    $editor = Complete-RegionDrag
    Wait-Reports 1
    'Passed: physical monitor bounds, dimmed background, overlapping hotkey rejection, and direct editor handoff.'
    if ($UseFunctionKeys) { 'Passed: frozen dropdown pixel with the configured function-key chord.' }

    if ($TestClipboard) {
        Wait-For {
            [QaptureProbe]::GetForegroundWindow() -eq $editor -and
            [QaptureProbe]::IsWindowVisible($editor)
        } 'editor focus' | Out-Null
        Wait-For {
            [QaptureProbe]::Key(0x11, $true)
            [QaptureProbe]::Tap(0x43)
            [QaptureProbe]::Key(0x11, $false)
            Start-Sleep -Milliseconds 160
            ([QaptureProbe]::ClipboardPixel($regionWidth, $regionHeight, 8, 12)) -eq 0x345191
        } 'editor clipboard output' | Out-Null
        if ($UseFunctionKeys) {
            Assert (([QaptureProbe]::ClipboardPixel($regionWidth, $regionHeight, ($menuX - $regionX), ($menuY - $regionY))) -eq $menuColor) 'Captured dropdown pixel did not match.'
        }
    }
    [QaptureProbe]::PostMessage($editor, 0x10, [UIntPtr]::Zero, [IntPtr]::Zero) | Out-Null
    Wait-For { ![QaptureProbe]::IsWindowVisible($editor) } 'editor close' | Out-Null

    if ($TestClipboard) {
        [QaptureProbe]::Shortcut($fullKey, $UseFunctionKeys)
        Wait-Reports 2
        Assert (([QaptureProbe]::ClipboardPixel(($right - $left), ($bottom - $top), ($regionX - $left), ($regionY - $top))) -eq 0x345191) 'Full desktop clipboard pixel did not match.'
        'Passed: native region and full desktop clipboard dimensions and fixture pixels.'
        $completed = 2
    } else {
        Assert ([QaptureProbe]::OpenClipboard($fixture.Handle)) 'Could not reserve clipboard for the full desktop test.'
        try {
            [QaptureProbe]::Shortcut($fullKey, $UseFunctionKeys)
            Wait-Reports 2
        } finally { [QaptureProbe]::CloseClipboard() | Out-Null }
        $report = Read-Entries | Where-Object event -eq 'capture-finished' | Select-Object -Last 1
        Assert ($report.outcome -eq 'error' -and $null -ne $report.clipboardMs) 'Expected a clipboard contention error after capturing the full desktop.'
        'Passed: full-desktop clipboard contention and cleanup.'
        $completed = 2
    }
    for ($sample = 1; $sample -le $Samples; $sample++) {
        $null = Open-Overlay
        [QaptureProbe]::Tap(0x1b)
        Wait-Reports ($completed + $sample)
    }
    $adjacent = $windows | Where-Object { $_.Right -eq $fixture.Left -and $_.Height -ge 240 } | Select-Object -First 1
    if ($adjacent) {
        $sharedTop = [Math]::Max($adjacent.Top, $fixture.Top)
        $sharedBottom = [Math]::Min($adjacent.Bottom, $fixture.Bottom)
        if ($sharedBottom - $sharedTop -ge 240) {
            $seam = $fixture.Left
            $y = $sharedTop + 140
            $adjacent.TopMost = $true
            $fixture.TopMost = $true
            [Windows.Forms.Application]::DoEvents()
            [QaptureProbe]::DwmFlush() | Out-Null
            $leftColor = [QaptureProbe]::ScreenColor(($seam - 20), ($y + 20))
            $rightColor = [QaptureProbe]::ScreenColor(($seam + 20), ($y + 20))
            Assert ($leftColor -eq 0x345191 -and $rightColor -eq 0x345191) 'The fixture was covered at the monitor seam.'
            $null = Open-Overlay
            Start-RegionDrag ($seam - 70) $y 140 60
            Wait-For {
                [QaptureProbe]::ScreenColor(($seam - 20), ($y + 20)) -eq $leftColor -and
                [QaptureProbe]::ScreenColor(($seam + 20), ($y + 20)) -eq $rightColor
            } 'a region spanning both displays' | Out-Null
            $editor = Complete-RegionDrag
            [QaptureProbe]::PostMessage($editor, 0x10, [UIntPtr]::Zero, [IntPtr]::Zero) | Out-Null
            Wait-For { ![QaptureProbe]::IsWindowVisible($editor) } 'editor close after cross-monitor selection' | Out-Null
            Wait-Reports ($completed + $Samples + 1)
            'Passed: exact selection pixels across the physical monitor seam.'
        }
    }
    $entries = @(Read-Entries)
    $warm = @($entries | Where-Object outcome -eq 'cancelled' | Select-Object -First $Samples | ForEach-Object overlayMs | Sort-Object)
    Assert ($warm.Count -eq $Samples) 'Expected all warm sessions to cancel and recover.'
    $p95 = $warm[[Math]::Ceiling(.95 * $warm.Count) - 1]
    'Warm overlay: p95 {0:N1} ms, max {1:N1} ms, n={2}' -f $p95, $warm[-1], $warm.Count
    'Dashboard startup: {0:N1} ms from launch, {1:N1} ms from Rust entry, n=1' -f $startup.Elapsed.TotalMilliseconds, ($entries | Where-Object event -eq 'dashboard-ready').startupMs
    Assert (![QaptureProbe]::RegisterHotKey($fixture.Handle, 37, $modifiers, $regionKey)) 'The app did not reserve its expected global shortcut.'
    $second = Start-Process -FilePath $exe -PassThru
    Assert ($second.WaitForExit(5000) -and !$app.HasExited) 'Second launch did not defer to the running app.'
    Wait-For {
        $app.Refresh()
        $app.MainWindowHandle -ne [IntPtr]::Zero -and [QaptureProbe]::IsWindowVisible($app.MainWindowHandle)
    } 'dashboard restoration by the first instance' | Out-Null
    'Passed: Windows shortcut reservation, close-to-tray, and single-instance behavior.'
} finally {
    [QaptureProbe]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero)
    if ($dropdown) { $dropdown.Dispose() }
    foreach ($window in $windows) { $window.Dispose() }
    foreach ($process in @($second, $app)) {
        if ($process -and !$process.HasExited) { Stop-Process -Id $process.Id -ErrorAction SilentlyContinue }
    }
    [QaptureProbe]::SetPhysicalCursorPos($previousPointer.X, $previousPointer.Y) | Out-Null
    [QaptureProbe]::SetForegroundWindow($previousWindow) | Out-Null
    [QaptureProbe]::SetThreadDpiAwarenessContext($previousDpi) | Out-Null
    $env:QAPTURE_DIAGNOSTICS = $previousDiagnostics
    "Timing log: $log"
}
