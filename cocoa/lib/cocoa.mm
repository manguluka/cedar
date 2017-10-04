
#import <Cocoa/Cocoa.h>

#include <iostream>
#include <string>
#include <thread>
#include <unordered_map>

#include <json/json.hpp>

using json = nlohmann::json;

@interface WindowDelegate : NSObject <NSWindowDelegate>
@end

@implementation WindowDelegate

- (NSSize)windowWillResize:(NSWindow *)__unused sender toSize:(NSSize)size {
    printf("Resized Window!: %f %f\n", size.width, size.height);
    fflush(stdout);

    return size;
}

@end

@interface Action : NSObject {
    std::string identifier;
}
@end

@implementation Action

- (id)initWithID:(std::string)ident {
    if (self = [super init]) {
        self->identifier = ident;
    }
    return self;
}

- (void)click:(id)__unused sender {
    // TODO: formalize as JSON
    std::cout << "click." << self->identifier << std::endl;
}

@end

extern "C" void run() {
    @autoreleasepool {
        [NSApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];

        // build menu for window
        {
            auto menubar = [NSMenu new];

            auto app_menu_item = [NSMenuItem new];
            [menubar addItem:app_menu_item];

            [NSApp setMainMenu:menubar];

            auto app_menu = [NSMenu new];
            auto quit_item = [[NSMenuItem alloc] initWithTitle:@"Quit"
                                                        action:@selector(terminate:)
                                                 keyEquivalent:@"q"];
            [app_menu addItem:quit_item];
            [app_menu_item setSubmenu:app_menu];
        }

        auto frame = NSMakeRect(0, 0, 500, 500);

#if (MAC_OS_X_VERSION_MAX_ALLOWED >= MAC_OS_X_VERSION_10_12)
        auto styleMask = NSWindowStyleMaskResizable | NSWindowStyleMaskTitled |
                         NSWindowStyleMaskMiniaturizable | NSWindowStyleMaskClosable;
#else
        auto styleMask = NSResizableWindowMask | NSTitledWindowMask | NSMiniaturizableWindowMask |
                         NSClosableWindowMask;
#endif

        auto window = [[NSWindow alloc] initWithContentRect:frame
                                                  styleMask:styleMask
                                                    backing:NSBackingStoreBuffered
                                                      defer:NO];

        window.delegate = [[WindowDelegate alloc] init];

        [window cascadeTopLeftFromPoint:NSMakePoint(0, 0)];
        [window center];
        [window setTitle:@"** cedar **"];
        [window makeKeyAndOrderFront:nil];

        // Bring window to front
        auto app = [NSRunningApplication currentApplication];
        [app activateWithOptions:NSApplicationActivateIgnoringOtherApps];

        std::thread([&] {
            std::unordered_map<std::string, NSView *> widgets;

            std::string line;
            while (std::getline(std::cin, line)) {
                // std::cerr << "received: " << line << std::endl;

                auto event = json::parse(line.c_str());

                if (event.count("Create")) {
                    // std::cerr << "received: " << event["Create"] << std::endl;

                    auto &create = event["Create"];
                    auto ident = create["id"];
                    auto widget = create["kind"];
                    std::string text = create["text"];

                    if (widget == "Button") {
                        static auto frame = NSMakeRect(0, 0, 100, 100);
                        frame.origin.x += 100; // HACK: avoid overlapping position

                        auto button = [[NSButton alloc] initWithFrame:frame];
                        button.bezelStyle = NSRoundedBezelStyle;

                        button.title = [NSString stringWithUTF8String:text.c_str()];

                        auto action = [[Action alloc] initWithID:ident];
                        [button setAction:@selector(click:)];
                        [button setTarget:action];

                        widgets[ident] = button;

                        [window.contentView addSubview:button];
                    } else if (widget == "Label") {
                        auto frame = NSMakeRect(100, 100, 100, 100);
                        auto label = [[NSTextField alloc] initWithFrame:frame];

                        [label setStringValue:[NSString stringWithUTF8String:text.c_str()]];

                        [label setBezeled:NO];
                        [label setDrawsBackground:NO];
                        [label setEditable:NO];
                        [label setSelectable:NO];

                        [label setAlignment:NSTextAlignmentCenter];

                        widgets[ident] = label;

                        [window.contentView addSubview:label];
                    }
                } else if (event.count("Update")) {
                    // std::cerr << "received: " << event["Update"] << std::endl;

                    auto &update = event["Update"];
                    auto ident = update[0];
                    auto attribute = update[1];
                    std::string value = update[2];

                    if (attribute == "Text") {
                        auto field = (NSTextField *)(widgets[ident]);
                        [field setStringValue:[NSString stringWithUTF8String:value.c_str()]];
                    }
                }
            }
        }).detach();

        [NSApp run];
    }
}
