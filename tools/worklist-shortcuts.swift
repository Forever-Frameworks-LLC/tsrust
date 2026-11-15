// Native process-boundary check. Run with the PID of an idle Worklist review window.
import AppKit
import ApplicationServices
import Foundation

func require(_ condition: Bool, _ message: String) throws {
    if !condition { throw NSError(domain: "Worklist shortcuts", code: 1, userInfo: [NSLocalizedDescriptionKey: message]) }
}
func attribute(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    return AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success ? value : nil
}
func find(_ element: AXUIElement, _ label: String) -> AXUIElement? {
    let name = attribute(element, "AXTitle") as? String ?? attribute(element, "AXDescription") as? String ?? ""
    if name == label { return element }
    for child in attribute(element, "AXChildren") as? [AXUIElement] ?? [] {
        if let found = find(child, label) { return found }
    }
    return nil
}
func height(_ element: AXUIElement) -> Double {
    var size = CGSize.zero
    guard let value = attribute(element, "AXSize"), CFGetTypeID(value) == AXValueGetTypeID(),
          AXValueGetValue(value as! AXValue, .cgSize, &size) else { return .nan }
    return size.height
}
func press(_ element: AXUIElement) throws {
    try require(AXUIElementPerformAction(element, kAXPressAction as CFString) == .success, "AXPress failed")
    Thread.sleep(forTimeInterval: 0.4)
}
func key(_ code: CGKeyCode, _ pid: pid_t) throws {
    try require(NSWorkspace.shared.frontmostApplication?.processIdentifier == pid, "Worklist lost focus; no keys sent")
    for down in [true, false] {
        let event = CGEvent(keyboardEventSource: nil, virtualKey: code, keyDown: down)
        event?.flags = .maskCommand
        event?.post(tap: .cghidEventTap)
    }
    Thread.sleep(forTimeInterval: 0.4)
}

do {
    try require(CommandLine.arguments.count == 2, "Usage: swift tools/worklist-shortcuts.swift WORKLIST_PID")
    guard let pid = pid_t(CommandLine.arguments[1]), let running = NSRunningApplication(processIdentifier: pid) else {
        throw NSError(domain: "Worklist shortcuts", code: 1, userInfo: [NSLocalizedDescriptionKey: "Missing process"])
    }
    try require(running.localizedName == "Worklist", "Target must be Worklist")
    let app = AXUIElementCreateApplication(pid)
    AXUIElementSetAttributeValue(app, "AXManualAccessibility" as CFString, kCFBooleanTrue)
    AXUIElementSetAttributeValue(app, "AXEnhancedUserInterface" as CFString, kCFBooleanTrue)
    running.activate(options: [])
    Thread.sleep(forTimeInterval: 0.4)
    var readyWindow: AXUIElement?
    for _ in 0..<30 {
        if let window = (attribute(app, "AXWindows") as? [AXUIElement])?.first,
           find(window, "New task") != nil, find(window, "Search tasks") != nil {
            readyWindow = window
            break
        }
        Thread.sleep(forTimeInterval: 0.1)
    }
    guard let window = readyWindow else {
        throw NSError(domain: "Worklist shortcuts", code: 1, userInfo: [NSLocalizedDescriptionKey: "Missing review controls"])
    }
    try require(find(window, "Task title") == nil, "Close any existing draft before this check")
    func searchHeight() throws -> Double {
        guard let search = find(window, "Search tasks") else { throw NSError(domain: "Worklist shortcuts", code: 1) }
        return height(search)
    }
    try key(29, pid) // Cmd+0
    let baseline = try searchHeight()
    var openedEmptySheet = false
    defer {
        if openedEmptySheet, let close = find(window, "Close") { try? press(close) }
        try? key(29, pid)
    }
    try key(24, pid) // Cmd+=
    try require(abs(try searchHeight() - baseline - 1.5) < 0.1, "Main-view zoom must fire exactly once")
    try key(29, pid)
    guard let create = find(window, "New task") else { throw NSError(domain: "Worklist shortcuts", code: 1) }
    try press(create)
    guard let title = find(window, "Task title"), let value = attribute(title, "AXValue") as? String else {
        throw NSError(domain: "Worklist shortcuts", code: 1, userInfo: [NSLocalizedDescriptionKey: "Task sheet did not open"])
    }
    try require(value.isEmpty, "Expected a new empty task sheet")
    openedEmptySheet = true
    try key(24, pid)
    try require(abs(try searchHeight() - baseline - 1.5) < 0.1, "Sheet-focused zoom must fire exactly once")
    try key(29, pid)
    try require(abs(try searchHeight() - baseline) < 0.1, "Sheet-focused reset must reach the application")
    print("PASS: main-view and sheet-focused shortcuts fire once; no task fields edited or saved")
} catch {
    fputs("FAIL: \(error.localizedDescription)\n", stderr)
    exit(1)
}
