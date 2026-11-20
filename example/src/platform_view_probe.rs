use std::{cell::Cell, rc::Rc};

#[cfg(target_os = "android")]
use jni::{
    JavaVM,
    objects::{JClass, JObject, JValue},
};
use tessera_components::md3::{
    modifier::ModifierExt as _, scrollable::ScrollableController, theme::material_theme,
};
use tessera_components::{boxed::boxed, modifier::ModifierExt as _};
use tessera_foundation::modifier::{ClickableArgs, ModifierExt as _};
#[cfg(target_os = "android")]
use tessera_ui::android::platform_views::AndroidPlatformViewFactoryHandle as ProbeFactory;
use tessera_ui::{
    Callback, Color, Dp, EntryPoint, FocusRequester, MeasurementError, Modifier, Plugin,
    PluginContext, PluginResult, PxPosition, State, key,
    layout::{LayoutResult, MeasureScope, layout},
    modifier::LayoutPolicy,
    platform_view, remember, tessera,
};

#[cfg(target_os = "ios")]
#[derive(Clone)]
struct ProbeFactory(usize);

struct ProbeFactories {
    first: ProbeFactory,
    second: ProbeFactory,
    editor: Option<ProbeFactory>,
    replacement: Option<ProbeFactory>,
    remount: ProbeFactory,
}

#[derive(Default)]
struct Probe {
    #[cfg(target_os = "android")]
    factories: Rc<std::cell::RefCell<Option<Rc<ProbeFactories>>>>,
    #[cfg(target_os = "android")]
    scroll: Rc<Cell<bool>>,
    #[cfg(target_os = "android")]
    keyboard: Rc<Cell<bool>>,
    #[cfg(target_os = "ios")]
    hit_test: Option<tessera_ui::Task>,
}

pub(crate) fn entry() -> EntryPoint {
    #[cfg(target_os = "ios")]
    if !std::env::args_os().any(|arg| arg == "--platform-view-probe") {
        return EntryPoint::new(crate::app::app);
    }
    let probe = Probe::default();
    #[cfg(target_os = "android")]
    let factories = probe.factories.clone();
    #[cfg(target_os = "ios")]
    let factories = Rc::new(ProbeFactories {
        first: ProbeFactory(0),
        second: ProbeFactory(1),
        editor: Some(ProbeFactory(2)),
        replacement: std::env::args_os()
            .any(|arg| arg == "--platform-view-replace")
            .then_some(ProbeFactory(3)),
        remount: ProbeFactory(42),
    });
    #[cfg(target_os = "android")]
    let scroll = probe.scroll.clone();
    #[cfg(target_os = "android")]
    let keyboard = probe.keyboard.clone();
    #[cfg(target_os = "ios")]
    let keyboard = std::env::args_os().any(|arg| arg == "--platform-view-keyboard");
    #[cfg(target_os = "ios")]
    let scroll = std::env::args_os().any(|arg| arg == "--platform-view-scroll");
    EntryPoint::new(move || {
        #[cfg(target_os = "android")]
        let Some(factories) = factories.borrow().clone() else {
            crate::app::app();
            return;
        };
        #[cfg(target_os = "android")]
        let (keyboard, scroll) = (keyboard.get(), scroll.get());
        if keyboard {
            keyboard_scene(
                factories
                    .editor
                    .clone()
                    .expect("native keyboard probe editor"),
            );
        } else if scroll {
            scroll_scene(factories.first.clone());
        } else {
            scene(factories.clone());
        }
    })
    .plugin(probe)
}

impl Plugin for Probe {
    #[cfg(target_os = "android")]
    fn on_resumed(&mut self, context: &PluginContext) -> PluginResult {
        if self.factories.borrow().is_some() {
            return Ok(());
        }
        let app = context.android_app();
        // SAFETY: AndroidApp keeps the VM and Activity references live for this hook.
        let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }?;
        let mut env = vm.attach_current_thread()?;
        let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
        let intent = env
            .call_method(&activity, "getIntent", "()Landroid/content/Intent;", &[])?
            .l()?;
        let extra = env.new_string("tessera.platform_view_probe")?;
        if !env
            .call_method(
                &intent,
                "getBooleanExtra",
                "(Ljava/lang/String;Z)Z",
                &[JValue::Object(extra.as_ref()), JValue::Bool(0)],
            )?
            .z()?
        {
            return Ok(());
        }

        let loader = env
            .call_method(
                &activity,
                "getClassLoader",
                "()Ljava/lang/ClassLoader;",
                &[],
            )?
            .l()?;
        let name = env.new_string("com.tessera.example.PlatformViewProbeFactory")?;
        let factory_class = JClass::from(
            env.call_method(
                &loader,
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                &[JValue::Object(name.as_ref())],
            )?
            .l()?,
        );
        let keyboard_extra = env.new_string("tessera.platform_view_keyboard")?;
        self.keyboard.set(
            env.call_method(
                &intent,
                "getBooleanExtra",
                "(Ljava/lang/String;Z)Z",
                &[JValue::Object(keyboard_extra.as_ref()), JValue::Bool(0)],
            )?
            .z()?,
        );
        let editor_extra = env.new_string("tessera.platform_view_editor")?;
        let editor = env
            .call_method(
                &intent,
                "getBooleanExtra",
                "(Ljava/lang/String;Z)Z",
                &[JValue::Object(editor_extra.as_ref()), JValue::Bool(0)],
            )?
            .z()?
            || self.keyboard.get();
        let scroll_extra = env.new_string("tessera.platform_view_scroll")?;
        self.scroll.set(
            env.call_method(
                &intent,
                "getBooleanExtra",
                "(Ljava/lang/String;Z)Z",
                &[JValue::Object(scroll_extra.as_ref()), JValue::Bool(0)],
            )?
            .z()?,
        );
        let replace_extra = env.new_string("tessera.platform_view_replace")?;
        let replacement = env
            .call_method(
                &intent,
                "getBooleanExtra",
                "(Ljava/lang/String;Z)Z",
                &[JValue::Object(replace_extra.as_ref()), JValue::Bool(0)],
            )?
            .z()?;
        let mut factory = |index| -> jni::errors::Result<_> {
            let factory = env.new_object(
                &factory_class,
                "(Landroid/content/Context;I)V",
                &[JValue::Object(&activity), JValue::Int(index)],
            )?;
            Ok(ProbeFactory::new(env.new_global_ref(factory)?))
        };
        *self.factories.borrow_mut() = Some(Rc::new(ProbeFactories {
            first: factory(0)?,
            second: factory(1)?,
            editor: editor.then(|| factory(2)).transpose()?,
            replacement: replacement.then(|| factory(3)).transpose()?,
            remount: factory(42)?,
        }));
        Ok(())
    }

    #[cfg(target_os = "ios")]
    fn on_resumed(&mut self, context: &PluginContext) -> PluginResult {
        if self.hit_test.is_none()
            && std::env::args_os().any(|arg| arg == "--check-platform-view-hit-testing")
        {
            let root = context.ui_view().expect("probe has no UIKit root view");
            self.hit_test = Some(tessera_ui::spawn_local(async move {
                // Wait for composition, then ask UIKit for the actual touch targets.
                tessera_ui::with_frame_time(|_| ()).await;
                tessera_ui::with_frame_time(|_| ()).await;
                let report = ios_hit_test(&root);
                std::fs::write(
                    std::env::temp_dir().join("tessera-platform-view-hit-test.txt"),
                    report,
                )
                .expect("write the UIKit hit-testing results");
            }));
        }
        Ok(())
    }
}

#[cfg(target_os = "ios")]
fn ios_hit_test(root: &objc2_ui_kit::UIView) -> String {
    use std::fmt::Write;

    use objc2_ui_kit::{UIButton, UITextField};

    let mut report = String::new();
    for (x, y, expected) in [
        (30.0, 100.0, "Tessera"),
        (48.0, 100.0, "Native 0"),
        (240.0, 150.0, "Native 1"),
        (162.0, 205.0, "Tessera"),
    ] {
        let mut point = root.bounds().origin;
        point.x = x;
        point.y = y;
        let actual = match root.hitTest_withEvent(point, None) {
            Some(view) if std::ptr::eq(&*view, root) => "Tessera".into(),
            Some(view) => view
                .downcast::<UIButton>()
                .ok()
                .and_then(|button| button.currentTitle())
                .map_or_else(|| "Unexpected UIKit view".into(), |title| title.to_string()),
            None => "No hit".into(),
        };
        let status = if actual == expected { "PASS" } else { "FAIL" };
        writeln!(
            report,
            "{status} ({x}, {y}): {actual} (expected {expected})"
        )
        .unwrap();
    }
    let mut point = root.bounds().origin;
    point.x = 100.0;
    point.y = 375.0;
    let sizes = root
        .hitTest_withEvent(point, None)
        .and_then(|view| view.downcast::<UITextField>().ok())
        .map(|editor| {
            let local = editor.bounds();
            let displayed = editor.convertRect_toView(local, Some(root));
            (
                local.size.width,
                local.size.height,
                displayed.size.width,
                displayed.size.height,
            )
        });
    let status = if sizes == Some((192.0, 44.0, 240.0, 55.0)) {
        "PASS"
    } else {
        "FAIL"
    };
    writeln!(report, "{status} scaled editor (local width, local height, displayed width, displayed height): {sizes:?}").unwrap();
    report
}

#[cfg(target_os = "ios")]
objc2::define_class!(
    #[unsafe(super(objc2_ui_kit::UIButton, objc2_ui_kit::UIControl, objc2_ui_kit::UIView, objc2_ui_kit::UIResponder, objc2::runtime::NSObject))]
    #[name = "TesseraProbeButton"]
    #[ivars = (Cell<usize>, Cell<bool>)]
    struct NativeProbeButton;

    impl NativeProbeButton {
        #[unsafe(method(probeTapped))]
        fn tapped(&self) {
            use objc2::DefinedClass;

            let (index, pressed) = self.ivars();
            pressed.set(!pressed.get());
            self.setTitle_forState(
                Some(&probe_title(index.get(), pressed.get())),
                objc2_ui_kit::UIControlState::Normal,
            );
        }
    }
);

#[cfg(target_os = "ios")]
fn probe_title(index: usize, pressed: bool) -> objc2::rc::Retained<objc2_foundation::NSString> {
    let suffix = if pressed { " pressed" } else { "" };
    objc2_foundation::NSString::from_str(&format!("Native {index}{suffix}"))
}

#[cfg(target_os = "ios")]
impl tessera_ui::ios::platform_views::IosPlatformViewFactory for ProbeFactory {
    fn create(
        &self,
        main_thread: objc2::MainThreadMarker,
    ) -> objc2::rc::Retained<objc2_ui_kit::UIView> {
        use objc2_ui_kit::{
            UIColor, UIControlEvents, UIControlState, UITextBorderStyle, UITextField,
        };

        if self.0 == 2 {
            let editor = UITextField::new(main_thread);
            editor.setBorderStyle(UITextBorderStyle::RoundedRect);
            editor.setPlaceholder(Some(objc2_foundation::ns_string!("Native editor")));
            return editor.into_super().into_super();
        }

        let button = main_thread
            .alloc::<NativeProbeButton>()
            .set_ivars((Cell::new(self.0), Cell::new(false)));
        // SAFETY: The subclass uses UIButton's standard initializer and targets its own action.
        let button: objc2::rc::Retained<NativeProbeButton> = unsafe {
            let button: objc2::rc::Retained<NativeProbeButton> =
                objc2::msg_send![super(button), init];
            button.addTarget_action_forControlEvents(
                Some(&button),
                objc2::sel!(probeTapped),
                UIControlEvents::TouchUpInside,
            );
            button
        };
        let [r, g, b] = match self.0 {
            0 => [32, 200, 80],
            3 => [30, 160, 240],
            _ => [240, 150, 30],
        };
        let color = UIColor::colorWithRed_green_blue_alpha(
            f64::from(r) / 255.0,
            f64::from(g) / 255.0,
            f64::from(b) / 255.0,
            1.0,
        );
        button.setBackgroundColor(Some(&color));
        button.setTitle_forState(Some(&probe_title(self.0, false)), UIControlState::Normal);
        button.into_super().into_super().into_super()
    }
}

#[tessera]
fn scene(factories: Rc<ProbeFactories>) {
    let pressed = remember(|| false);
    let between_pressed = remember(|| false);
    let clipped_area_pressed = remember(|| false);
    let show_second = remember(|| true);
    let second_behind = remember(|| false);
    let second_removed = remember(|| false);
    let replaced = remember(|| false);
    let remounted = remember(|| false);
    boxed()
        .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
        .children(move || {
            if cfg!(target_os = "android") && second_behind.get() {
                patch(region(304.0, 264.0, 24.0, 20.0), Color::BLACK);
            }
            patch(
                region(24.0, 80.0, 24.0, 40.0).clickable(ClickableArgs {
                    on_click: Callback::new(move || clipped_area_pressed.set(true)),
                    label: Some("Clipped native view underlay".into()),
                    ..Default::default()
                }),
                if clipped_area_pressed.get() {
                    Color::BLACK
                } else {
                    Color::WHITE
                },
            );
            let behind = second_behind.get();
            for layer in if behind { [2, 0, 1] } else { [0, 1, 2] } {
                key(layer, || match layer {
                    0 => {
                        let first = factories.first.clone();
                        boxed()
                            .modifier(region(40.0, 80.0, 180.0, 144.0).clip_to_bounds())
                            .children(move || {
                                platform_view(first.clone(), region(-16.0, 0.0, 180.0, 144.0));
                            });
                    }
                    1 => {
                        let between = region(64.0, 136.0, 194.0, 44.0);
                        let between = if cfg!(target_os = "android") {
                            between.clickable(ClickableArgs {
                                on_click: Callback::new(move || {
                                    between_pressed.with_mut(|value| *value = !*value)
                                }),
                                label: Some("Tessera middle probe".into()),
                                ..Default::default()
                            })
                        } else {
                            between
                        };
                        patch(
                            between,
                            if between_pressed.get() {
                                Color::BLACK
                            } else {
                                Color::from_rgb_u8(171, 55, 191)
                            },
                        );
                    }
                    _ if !second_removed.get() => {
                        let (item, factory) = if replaced.get() {
                            (
                                3,
                                factories
                                    .replacement
                                    .clone()
                                    .expect("replacement factory enabled"),
                            )
                        } else if remounted.get() {
                            (42, factories.remount.clone())
                        } else {
                            (1, factories.second.clone())
                        };
                        key(item, || {
                            second_native_probe(factory, show_second, behind);
                        });
                    }
                    _ => {}
                });
            }
            if replaced.get() {
                patch(region(304.0, 264.0, 24.0, 20.0), Color::BLACK);
            }
            patch(
                region(140.0, 192.0, 92.0, 40.0).clickable(ClickableArgs {
                    on_click: Callback::new(move || pressed.with_mut(|value| *value = !*value)),
                    label: Some(if pressed.get() {
                        "Tessera foreground pressed".into()
                    } else {
                        "Tessera foreground probe".into()
                    }),
                    ..Default::default()
                }),
                if pressed.get() {
                    Color::from_rgb_u8(214, 35, 51)
                } else {
                    Color::from_rgb_u8(31, 93, 223)
                },
            );
            patch(
                region(24.0, 280.0, 96.0, 44.0).clickable(ClickableArgs {
                    on_click: Callback::new(move || show_second.with_mut(|value| *value = !*value)),
                    label: Some("Toggle second native probe".into()),
                    ..Default::default()
                }),
                Color::from_rgb_u8(40, 40, 40),
            );
            patch(
                region(132.0, 280.0, 96.0, 44.0).clickable(ClickableArgs {
                    on_click: Callback::new(move || {
                        second_behind.with_mut(|value| *value = !*value)
                    }),
                    label: Some("Move and reorder second native probe".into()),
                    ..Default::default()
                }),
                Color::from_rgb_u8(60, 60, 60),
            );
            if !second_removed.get() && !replaced.get() {
                let replacing = factories.replacement.is_some();
                patch(
                    region(248.0, 280.0, 96.0, 44.0).clickable(ClickableArgs {
                        on_click: Callback::new(move || {
                            if replacing {
                                replaced.set(true);
                            } else {
                                second_removed.set(true);
                            }
                        }),
                        label: Some(if replacing {
                            "Replace second native probe".into()
                        } else {
                            "Remove second native probe".into()
                        }),
                        ..Default::default()
                    }),
                    Color::from_rgb_u8(80, 80, 80),
                );
            }
            if show_second.get()
                && !second_removed.get()
                && factories.replacement.is_none()
                && !remounted.get()
            {
                patch(
                    region(248.0, 328.0, 96.0, 40.0).clickable(ClickableArgs {
                        on_click: Callback::new(move || {
                            remounted.set(true);
                            show_second.set(false);
                        }),
                        label: Some("Remount second native probe".into()),
                        ..Default::default()
                    }),
                    Color::from_rgb_u8(80, 80, 80),
                );
            }
            if let Some(editor) = factories.editor.clone() {
                boxed()
                    .modifier(Modifier::new().offset(Dp(40.0), Dp(350.0)))
                    .children(move || {
                        let editor = editor.clone();
                        tessera_ui::scaled_box(1.25).child(move || {
                            platform_view(
                                editor.clone(),
                                Modifier::new().size(Dp(192.0), Dp(44.0)),
                            );
                        });
                    });
            }
        });
}

#[tessera]
fn keyboard_scene(#[prop(skip_eq)] editor: ProbeFactory) {
    boxed()
        .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
        .children(move || {
            keyboard_target("Before native editor".to_owned(), 80.0);
            platform_view(editor.clone(), region(40.0, 140.0, 208.0, 44.0));
            keyboard_target("After native editor".to_owned(), 200.0);
        });
}

#[tessera]
fn keyboard_target(label: String, y: f64) {
    let focus = remember(FocusRequester::new).get();
    let activated = remember(|| false);
    patch(
        region(40.0, y, 208.0, 44.0).clickable(ClickableArgs {
            on_click: Callback::new(move || activated.with_mut(|value| *value = !*value)),
            label: Some(label.into()),
            focus_requester: Some(focus),
            ..Default::default()
        }),
        if activated.get() {
            Color::WHITE
        } else if focus.is_focused() {
            Color::from_rgb_u8(0, 255, 0)
        } else {
            Color::BLACK
        },
    );
}

#[tessera]
fn scroll_scene(#[prop(skip_eq)] factory: ProbeFactory) {
    let scroll = remember(ScrollableController::new);
    material_theme().child(move || {
        let factory = factory.clone();
        boxed()
            .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
            .children(move || {
                let factory = factory.clone();
                boxed()
                    .modifier(region(40.0, 80.0, 250.0, 144.0).vertical_scroll(scroll))
                    .children(move || {
                        let factory = factory.clone();
                        boxed()
                            .modifier(Modifier::new().size(Dp(250.0), Dp(244.0)))
                            .children(move || {
                                platform_view(factory.clone(), region(24.0, 114.0, 180.0, 86.0));
                                patch(
                                    region(80.0, 130.0, 56.0, 24.0),
                                    Color::from_rgb_u8(171, 55, 191),
                                );
                            });
                    });
            });
    });
}

#[tessera]
fn second_native_probe(#[prop(skip_eq)] factory: ProbeFactory, visible: State<bool>, behind: bool) {
    let (x, y) = if cfg!(target_os = "android") {
        (if behind { 124.5 } else { 112.0 }, 112.0)
    } else {
        (112.0, if behind { 124.5 } else { 112.0 })
    };
    layout(PlaceWhenVisible(visible))
        .modifier(Modifier::new().offset(Dp(x), Dp(y)))
        .child(move || {
            if cfg!(target_os = "android") {
                let factory = factory.clone();
                tessera_ui::scaled_box(2.0).child(move || {
                    platform_view(factory.clone(), Modifier::new().size(Dp(84.0), Dp(72.0)));
                });
            } else {
                platform_view(factory.clone(), Modifier::new().size(Dp(168.0), Dp(144.0)));
            }
        });
}

#[derive(Clone, PartialEq)]
struct PlaceWhenVisible(State<bool>);

impl LayoutPolicy for PlaceWhenVisible {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let constraint = *scope.parent_constraint().as_ref();
        let child = scope.children()[0].measure(scope, &constraint)?;
        let mut result = LayoutResult::new(child.size());
        if self.0.get() {
            result.place_child(child, PxPosition::ZERO);
        }
        Ok(result)
    }
}

fn region(x: f64, y: f64, width: f64, height: f64) -> Modifier {
    Modifier::new()
        .offset(Dp(x), Dp(y))
        .size(Dp(width), Dp(height))
}

#[tessera]
fn patch(modifier: Modifier, color: Color) {
    boxed().modifier(modifier.background(color));
}
