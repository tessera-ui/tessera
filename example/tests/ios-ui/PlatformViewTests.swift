import UIKit
import XCTest

final class PlatformViewTests: XCTestCase {
    func testNativeAndTesseraAccessibilityAndTouch() {
        continueAfterFailure = false
        let device = XCUIDevice.shared
        device.orientation = .portrait
        addTeardownBlock { device.orientation = .portrait }
        let app = XCUIApplication(bundleIdentifier: "com.tessera.example.platform-view-probe")
        app.launchArguments = ["--platform-view-probe"]
        app.launch()
        assertOrientation(of: app, landscape: false)
        let native = app.buttons["Native 1"]
        let foreground = app.buttons["Tessera foreground probe"]
        let toggle = app.buttons["Toggle second native probe"]
        XCTAssertTrue(native.waitForExistence(timeout: 5), app.debugDescription)
        XCTAssertEqual(app.buttons.matching(identifier: "Native 1").count, 1)
        XCTAssertTrue(foreground.exists)
        XCTAssertTrue(toggle.exists)
        assertPixels(in: app, [
            (30, 100, [255, 255, 255]), // clipped-out part of the first view
            (48, 100, [32, 200, 80]),
            (80, 150, [214, 128, 224]),
            (240, 150, [240, 150, 30]),
            (162, 205, [98, 163, 240]),
        ])

        func tap(_ x: CGFloat, _ y: CGFloat) {
            app.coordinate(withNormalizedOffset: .zero)
                .withOffset(CGVector(dx: x, dy: y)).tap()
        }
        tap(270, 200)
        let clicked = app.buttons["Native 1 pressed"]
        XCTAssertTrue(clicked.waitForExistence(timeout: 3))
        tap(162, 205)
        XCTAssertTrue(app.buttons["Tessera foreground pressed"].waitForExistence(timeout: 3))
        XCTAssertTrue(clicked.exists, "Tessera's overlay tap also activated the native button")

        tap(44, 300)
        XCTAssertTrue(clicked.waitForNonExistence(timeout: 3))
        assertPixels(in: app, [(240, 150, [214, 128, 224])])
        tap(44, 300)
        XCTAssertTrue(clicked.waitForExistence(timeout: 3), "Hiding discarded the native view")
        assertPixels(in: app, [(240, 150, [240, 150, 30])])

        device.orientation = .landscapeLeft
        assertOrientation(of: app, landscape: true)
        XCTAssertTrue(clicked.exists, "Resizing discarded the native view")
        assertPixels(in: app, [(80, 150, [214, 128, 224]), (240, 150, [240, 150, 30])],
                     screenIsRotatedLeft: true)
        tap(270, 200)
        XCTAssertTrue(native.waitForExistence(timeout: 3), "Landscape touch missed the native view")
        tap(270, 200)
        XCTAssertTrue(clicked.waitForExistence(timeout: 3))
        device.orientation = .portrait
        assertOrientation(of: app, landscape: false)

        let editor = app.textFields["Native editor"]
        XCTAssertTrue(editor.exists)
        editor.tap()
        editor.typeText("Hello")
        XCTAssertEqual(editor.value as? String, "Hello")

        XCUIDevice.shared.press(.home)
        app.activate()
        XCTAssertTrue(clicked.waitForExistence(timeout: 5), "Resuming discarded the native view")
        XCTAssertEqual(editor.value as? String, "Hello")
        tap(268, 344)
        XCTAssertTrue(clicked.waitForNonExistence(timeout: 3))
        assertPixels(in: app, [(240, 150, [214, 128, 224])])
        tap(44, 300)
        let remounted = app.buttons["Native 42"]
        XCTAssertTrue(remounted.waitForExistence(timeout: 3), "The new keyed view did not start with fresh state")
        XCTAssertFalse(native.exists)
        tap(270, 200)
        let remountedPressed = app.buttons["Native 42 pressed"]
        XCTAssertTrue(remountedPressed.waitForExistence(timeout: 3))
        tap(268, 300)
        XCTAssertTrue(remountedPressed.waitForNonExistence(timeout: 3))
        assertPixels(in: app, [(240, 150, [214, 128, 224])])
        tap(44, 300)
        tap(44, 300)
        XCTAssertFalse(remounted.exists || remountedPressed.exists,
                       "A released view must not be restored by visibility changes")
    }

    func testReorderMovesNativePixelsAndHitRegionsTogether() {
        continueAfterFailure = false
        XCUIDevice.shared.orientation = .portrait
        let app = XCUIApplication(bundleIdentifier: "com.tessera.example.platform-view-probe")
        app.launchArguments = ["--platform-view-probe"]
        app.launch()
        assertOrientation(of: app, landscape: false)

        let reorder = app.buttons["Move and reorder second native probe"]
        let firstPressed = app.buttons["Native 0 pressed"]
        let secondPressed = app.buttons["Native 1 pressed"]
        XCTAssertTrue(reorder.waitForExistence(timeout: 5))
        func at(_ x: CGFloat, _ y: CGFloat) -> XCUICoordinate {
            app.coordinate(withNormalizedOffset: .zero).withOffset(CGVector(dx: x, dy: y))
        }

        assertPixels(in: app, [(190, 130, [240, 150, 30]), (240, 150, [240, 150, 30]),
                               (270, 262, [255, 255, 255])])
        at(190, 130).tap()
        XCTAssertTrue(secondPressed.waitForExistence(timeout: 3))
        XCTAssertFalse(firstPressed.exists)

        reorder.tap()
        assertPixels(in: app, [(190, 130, [32, 200, 80]), (240, 150, [214, 128, 224]),
                               (270, 262, [240, 150, 30])])
        XCTAssertTrue(secondPressed.exists, "Reordering discarded the native view")
        at(190, 130).tap()
        XCTAssertTrue(firstPressed.waitForExistence(timeout: 3))
        XCTAssertTrue(secondPressed.exists, "The covered view also received touch")
        at(270, 262).tap()
        XCTAssertTrue(app.buttons["Native 1"].waitForExistence(timeout: 3))

        reorder.tap()
        assertPixels(in: app, [(190, 130, [240, 150, 30]), (240, 150, [240, 150, 30]),
                               (270, 262, [255, 255, 255])])
        at(190, 130).tap()
        XCTAssertTrue(secondPressed.waitForExistence(timeout: 3))
        XCTAssertTrue(firstPressed.exists, "Returning the upper view also activated the lower one")
    }

    func testReplacingKeyedViewMovesPixelsAccessibilityAndTouchToNewIdentity() {
        continueAfterFailure = false
        XCUIDevice.shared.orientation = .portrait
        let app = XCUIApplication(bundleIdentifier: "com.tessera.example.platform-view-probe")
        app.launchArguments = ["--platform-view-probe", "--platform-view-replace"]
        app.launch()
        assertOrientation(of: app, landscape: false)
        let old = app.buttons["Native 1"]
        let replacement = app.buttons["Native 3"]
        XCTAssertTrue(old.waitForExistence(timeout: 5))
        XCTAssertFalse(replacement.exists)
        assertPixels(in: app, [(240, 150, [240, 150, 30]), (316, 274, [255, 255, 255])])
        app.buttons["Replace second native probe"].tap()
        XCTAssertTrue(replacement.waitForExistence(timeout: 3))
        XCTAssertFalse(old.exists)
        assertPixels(in: app, [(240, 150, [30, 160, 240]), (316, 274, [0, 0, 0])])
        app.coordinate(withNormalizedOffset: .zero)
            .withOffset(CGVector(dx: 270, dy: 200)).tap()
        XCTAssertTrue(app.buttons["Native 3 pressed"].waitForExistence(timeout: 3))
        XCTAssertFalse(app.buttons["Native 1 pressed"].exists)
    }

    func testHardwareTabMovesAcrossTesseraAndNativeEditor() {
        continueAfterFailure = false
        XCUIDevice.shared.orientation = .portrait
        let app = XCUIApplication(bundleIdentifier: "com.tessera.example.platform-view-probe")
        app.launchArguments = ["--platform-view-probe", "--platform-view-keyboard"]
        app.launch()
        let editor = app.textFields["Native editor"]
        XCTAssertTrue(editor.waitForExistence(timeout: 5))

        app.typeKey(XCUIKeyboardKey.tab.rawValue, modifierFlags: [])
        assertPixels(in: app, [(44, 85, [0, 255, 0]), (44, 205, [0, 0, 0])])
        app.typeKey(XCUIKeyboardKey.tab.rawValue, modifierFlags: [])
        editor.typeText("Hello")
        XCTAssertEqual(editor.value as? String, "Hello")
        app.typeKey(XCUIKeyboardKey.tab.rawValue, modifierFlags: [])
        assertPixels(in: app, [(44, 85, [0, 0, 0]), (44, 205, [0, 255, 0])])

        app.typeKey(XCUIKeyboardKey.tab.rawValue, modifierFlags: .shift)
        editor.typeText("!")
        XCTAssertEqual(editor.value as? String, "Hello!")
        app.typeKey(XCUIKeyboardKey.tab.rawValue, modifierFlags: .shift)
        assertPixels(in: app, [(44, 85, [0, 255, 0]), (44, 205, [0, 0, 0])])
        app.typeKey(XCUIKeyboardKey.space.rawValue, modifierFlags: [])
        assertPixels(in: app, [(44, 85, [255, 255, 255]), (44, 205, [0, 0, 0])])
    }

    func testScrollMovesNativePixelsAndHitRegionTogether() {
        continueAfterFailure = false
        let device = XCUIDevice.shared
        device.orientation = .portrait
        let app = XCUIApplication(bundleIdentifier: "com.tessera.example.platform-view-probe")
        app.launchArguments = ["--platform-view-probe", "--platform-view-scroll"]
        app.launch()
        assertOrientation(of: app, landscape: false)
        let native = app.buttons["Native 0"]
        let pressed = app.buttons["Native 0 pressed"]
        XCTAssertTrue(native.waitForExistence(timeout: 5))
        let originalTop = native.frame.minY
        assertPixels(in: app, [
            (80, 204, [32, 200, 80]), (142, 217, [214, 128, 224]),
            (80, 232, [255, 255, 255]),
        ])

        func at(_ x: CGFloat, _ y: CGFloat) -> XCUICoordinate {
            app.coordinate(withNormalizedOffset: .zero)
                .withOffset(CGVector(dx: x, dy: y))
        }

        at(276, 208).press(forDuration: 0.05, thenDragTo: at(276, 86))
        let scrolled = XCTNSPredicateExpectation(predicate: NSPredicate { _, _ in
            native.exists && native.frame.minY < originalTop - 92
        }, object: nil)
        XCTAssertEqual(XCTWaiter.wait(for: [scrolled], timeout: 5), .completed)
        assertPixels(in: app, [
            (80, 105, [32, 200, 80]), (142, 117, [214, 128, 224]),
            (80, 204, [255, 255, 255]),
        ])
        at(80, 204).tap()
        XCTAssertFalse(pressed.exists, "The native view retained its old hit region")
        at(80, 105).tap()
        XCTAssertTrue(pressed.waitForExistence(timeout: 3))

        at(276, 86).press(forDuration: 0.05, thenDragTo: at(276, 208))
        let returned = XCTNSPredicateExpectation(predicate: NSPredicate { _, _ in
            pressed.exists && abs(pressed.frame.minY - originalTop) < 2
        }, object: nil)
        XCTAssertEqual(XCTWaiter.wait(for: [returned], timeout: 5), .completed)
        assertPixels(in: app, [
            (80, 105, [255, 255, 255]), (80, 204, [32, 200, 80]),
            (142, 217, [214, 128, 224]),
        ])
    }

    private func assertOrientation(of app: XCUIApplication, landscape: Bool) {
        let resized = XCTNSPredicateExpectation(predicate: NSPredicate { _, _ in
            let frame = app.frame
            return (frame.width > frame.height) == landscape && !frame.isEmpty
        }, object: nil)
        XCTAssertEqual(XCTWaiter.wait(for: [resized], timeout: 5), .completed)
    }

    private func assertPixels(
        in app: XCUIApplication, _ samples: [(CGFloat, CGFloat, [Int])],
        screenIsRotatedLeft: Bool = false,
        file: StaticString = #filePath, line: UInt = #line
    ) {
        // The system screenshot can be in fixed screen coordinates. App screenshots
        // on iOS 27 can retain rotated content even when the PNG dimensions are landscape.
        let screenshot = XCUIScreen.main.screenshot()
        guard let image = UIImage(data: screenshot.pngRepresentation)?.cgImage,
              let colorSpace = CGColorSpace(name: CGColorSpace.sRGB) else {
            return XCTFail("Could not read the system compositor screenshot", file: file, line: line)
        }
        let frame = app.frame
        let transpose = screenIsRotatedLeft && image.width < image.height
        let scaleX = CGFloat(transpose ? image.height : image.width) / frame.width
        let scaleY = CGFloat(transpose ? image.width : image.height) / frame.height
        for (x, y, expected) in samples {
            let px = (x * scaleX).rounded()
            let py = (y * scaleY).rounded()
            let rect = CGRect(x: transpose ? CGFloat(image.width - 1) - py : px,
                              y: transpose ? px : py, width: 1, height: 1)
            var rgba = [UInt8](repeating: 0, count: 4)
            let sampled = rgba.withUnsafeMutableBytes { bytes -> Bool in
                guard let pixel = image.cropping(to: rect),
                      let context = CGContext(data: bytes.baseAddress, width: 1, height: 1,
                                              bitsPerComponent: 8, bytesPerRow: 4, space: colorSpace,
                                              bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
                                                  | CGBitmapInfo.byteOrder32Big.rawValue) else { return false }
                context.draw(pixel, in: CGRect(x: 0, y: 0, width: 1, height: 1))
                return true
            }
            let actual = rgba.prefix(3).map(Int.init)
            if !sampled || !zip(actual, expected).allSatisfy({ abs($0 - $1) <= 12 }) {
                add(XCTAttachment(screenshot: screenshot))
                XCTFail("Screenshot \(image.width)×\(image.height), app \(frame), at (\(x), \(y)): \(actual), expected \(expected)",
                        file: file, line: line)
            }
        }
    }
}
