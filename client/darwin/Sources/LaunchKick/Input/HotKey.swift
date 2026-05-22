import AppKit
import Carbon

final class HotKey {
    private var hotKeyRef: EventHotKeyRef?
    private let handler: () -> Void

    init(handler: @escaping () -> Void) {
        self.handler = handler
    }

    func register() {
        var eventType = EventTypeSpec(
            eventClass: OSType(kEventClassKeyboard),
            eventKind: UInt32(kEventHotKeyPressed)
        )

        let selfPointer = Unmanaged.passUnretained(self).toOpaque()

        InstallEventHandler(
            GetApplicationEventTarget(),
            { _, _, userData in
                guard let userData else { return noErr }

                let hotKey = Unmanaged<HotKey>
                    .fromOpaque(userData)
                    .takeUnretainedValue()

                hotKey.handler()
                return noErr
            },
            1,
            &eventType,
            selfPointer,
            nil
        )

        let hotKeyID = EventHotKeyID(signature: fourCharCode("LNCH"), id: 1)

        RegisterEventHotKey(
            UInt32(kVK_Space),
            UInt32(optionKey),
            hotKeyID,
            GetApplicationEventTarget(),
            0,
            &hotKeyRef
        )
    }
}

private func fourCharCode(_ string: String) -> OSType {
    var result: UInt32 = 0

    for scalar in string.unicodeScalars.prefix(4) {
        result = (result << 8) + scalar.value
    }

    return OSType(result)
}
