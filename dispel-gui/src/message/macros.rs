/// Generates the `MessageExt` trait and its `impl` for `Message`.
///
/// The workspace shortcuts (`file_tree`, `tab_bar`) and the generic `editor`/`workspace`
/// methods are hardcoded. Pass one entry per editor shortcut method:
///
/// ```rust
/// define_message_ext! {
///     weapon: Weapon($crate::message::editor::weapon::WeaponEditorMessage),
///     …
/// }
/// ```
#[macro_export]
macro_rules! define_message_ext {
    ( $( $method:ident : $Variant:ident ( $MsgType:path ) ),+ $(,)? ) => {

        pub trait MessageExt {
            fn editor<M>(message: M) -> $crate::message::Message
            where
                M: ::std::convert::Into<$crate::message::EditorMessage>;

            fn workspace<M>(message: M) -> $crate::message::Message
            where
                M: ::std::convert::Into<$crate::message::WorkspaceMessage>;

            #[allow(dead_code)]
            fn file_tree(
                msg: $crate::components::file_tree::message::FileTreeMessage,
            ) -> $crate::message::Message;

            #[allow(dead_code)]
            fn tab_bar(
                msg: $crate::components::tab_bar::TabBarMessage,
            ) -> $crate::message::Message;

            $( #[allow(dead_code)] fn $method(msg: $MsgType) -> $crate::message::Message; )+
        }

        impl MessageExt for $crate::message::Message {
            fn editor<M>(message: M) -> $crate::message::Message
            where
                M: ::std::convert::Into<$crate::message::EditorMessage>,
            {
                $crate::message::Message::Editor(message.into())
            }

            fn workspace<M>(message: M) -> $crate::message::Message
            where
                M: ::std::convert::Into<$crate::message::WorkspaceMessage>,
            {
                $crate::message::Message::Workspace(message.into())
            }

            fn file_tree(
                msg: $crate::components::file_tree::message::FileTreeMessage,
            ) -> $crate::message::Message {
                $crate::message::Message::Workspace(
                    $crate::message::WorkspaceMessage::FileTree(msg),
                )
            }

            fn tab_bar(
                msg: $crate::components::tab_bar::TabBarMessage,
            ) -> $crate::message::Message {
                $crate::message::Message::Workspace(
                    $crate::message::WorkspaceMessage::TabBar(msg),
                )
            }

            $(
                fn $method(msg: $MsgType) -> $crate::message::Message {
                    $crate::message::Message::Editor(
                        $crate::message::EditorMessage::$Variant(msg),
                    )
                }
            )+
        }
    };
}
