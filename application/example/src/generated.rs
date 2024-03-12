use foliage::bevy_ecs;
use foliage::bevy_ecs::prelude::Resource;

#[foliage::assets(crate::Engen, "../assets/", "/foliage/demo/assets/")]
#[derive(Resource, Clone)]
pub(crate) struct AssetGen {
    #[bytes(path = "img.png", group = generated)]
    _id: AssetKey,
    #[icon(path = "icons/skip-forward.icon", opt = FeatherIcon::SkipForward)]
    _id: AssetKey,
    #[icon(path = "icons/shield-off.icon", opt = FeatherIcon::ShieldOff)]
    _id: AssetKey,
    #[icon(path = "icons/plus-circle.icon", opt = FeatherIcon::PlusCircle)]
    _id: AssetKey,
    #[icon(path = "icons/settings.icon", opt = FeatherIcon::Settings)]
    _id: AssetKey,
    #[icon(path = "icons/rotate-cw.icon", opt = FeatherIcon::RotateCW)]
    _id: AssetKey,
    #[icon(path = "icons/search.icon", opt = FeatherIcon::Search)]
    _id: AssetKey,
    #[icon(path = "icons/command.icon", opt = FeatherIcon::Command)]
    _id: AssetKey,
    #[icon(path = "icons/cloud-snow.icon", opt = FeatherIcon::CloudSnow)]
    _id: AssetKey,
    #[icon(path = "icons/file.icon", opt = FeatherIcon::File)]
    _id: AssetKey,
    #[icon(path = "icons/camera-off.icon", opt = FeatherIcon::CameraOff)]
    _id: AssetKey,
    #[icon(path = "icons/x.icon", opt = FeatherIcon::X)]
    _id: AssetKey,
    #[icon(path = "icons/navigation-2.icon", opt = FeatherIcon::NavigationTwo)]
    _id: AssetKey,
    #[icon(path = "icons/corner-right-up.icon", opt = FeatherIcon::CornerRightUp)]
    _id: AssetKey,
    #[icon(path = "icons/divide-circle.icon", opt = FeatherIcon::DivideCircle)]
    _id: AssetKey,
    #[icon(path = "icons/corner-down-left.icon", opt = FeatherIcon::CornerDownLeft)]
    _id: AssetKey,
    #[icon(path = "icons/check-circle.icon", opt = FeatherIcon::CheckCircle)]
    _id: AssetKey,
    #[icon(path = "icons/volume.icon", opt = FeatherIcon::Volume)]
    _id: AssetKey,
    #[icon(path = "icons/globe.icon", opt = FeatherIcon::Globe)]
    _id: AssetKey,
    #[icon(path = "icons/paperclip.icon", opt = FeatherIcon::Paperclip)]
    _id: AssetKey,
    #[icon(path = "icons/corner-right-down.icon", opt = FeatherIcon::CornerRightDown)]
    _id: AssetKey,
    #[icon(path = "icons/edit.icon", opt = FeatherIcon::Edit)]
    _id: AssetKey,
    #[icon(path = "icons/codesandbox.icon", opt = FeatherIcon::CodeSandbox)]
    _id: AssetKey,
    #[icon(path = "icons/mic.icon", opt = FeatherIcon::Mic)]
    _id: AssetKey,
    #[icon(path = "icons/tag.icon", opt = FeatherIcon::Tag)]
    _id: AssetKey,
    #[icon(path = "icons/smile.icon", opt = FeatherIcon::Smile)]
    _id: AssetKey,
    #[icon(path = "icons/arrow-up-left.icon", opt = FeatherIcon::ArrowUpLeft)]
    _id: AssetKey,
    #[icon(path = "icons/terminal.icon", opt = FeatherIcon::Terminal)]
    _id: AssetKey,
    #[icon(path = "icons/play.icon", opt = FeatherIcon::Play)]
    _id: AssetKey,
    #[icon(path = "icons/git-commit.icon", opt = FeatherIcon::GitCommit)]
    _id: AssetKey,
    #[icon(path = "icons/map-pin.icon", opt = FeatherIcon::MapPin)]
    _id: AssetKey,
    #[icon(path = "icons/message-circle.icon", opt = FeatherIcon::MessageCircle)]
    _id: AssetKey,
    #[icon(path = "icons/bold.icon", opt = FeatherIcon::Bold)]
    _id: AssetKey,
    #[icon(path = "icons/x-circle.icon", opt = FeatherIcon::XCircle)]
    _id: AssetKey,
    #[icon(path = "icons/aperture.icon", opt = FeatherIcon::Aperture)]
    _id: AssetKey,
    #[icon(path = "icons/trending-up.icon", opt = FeatherIcon::TrendingUp)]
    _id: AssetKey,
    #[icon(path = "icons/italic.icon", opt = FeatherIcon::Italic)]
    _id: AssetKey,
    #[icon(path = "icons/code.icon", opt = FeatherIcon::Code)]
    _id: AssetKey,
    #[icon(path = "icons/printer.icon", opt = FeatherIcon::Printer)]
    _id: AssetKey,
    #[icon(path = "icons/layout.icon", opt = FeatherIcon::Layout)]
    _id: AssetKey,
    #[icon(path = "icons/file-minus.icon", opt = FeatherIcon::FileMinus)]
    _id: AssetKey,
    #[icon(path = "icons/chevron-right.icon", opt = FeatherIcon::ChevronRight)]
    _id: AssetKey,
    #[icon(path = "icons/droplet.icon", opt = FeatherIcon::Droplet)]
    _id: AssetKey,
    #[icon(path = "icons/shopping-bag.icon", opt = FeatherIcon::ShoppingBag)]
    _id: AssetKey,
    #[icon(path = "icons/git-pull-request.icon", opt = FeatherIcon::GitPullRequest)]
    _id: AssetKey,
    #[icon(path = "icons/tablet.icon", opt = FeatherIcon::Tablet)]
    _id: AssetKey,
    #[icon(path = "icons/archive.icon", opt = FeatherIcon::Archive)]
    _id: AssetKey,
    #[icon(path = "icons/layers.icon", opt = FeatherIcon::Layers)]
    _id: AssetKey,
    #[icon(path = "icons/link.icon", opt = FeatherIcon::Link)]
    _id: AssetKey,
    #[icon(path = "icons/triangle.icon", opt = FeatherIcon::Triangle)]
    _id: AssetKey,
    #[icon(path = "icons/feather.icon", opt = FeatherIcon::Feather)]
    _id: AssetKey,
    #[icon(path = "icons/arrow-down-right.icon", opt = FeatherIcon::ArrowDownRight)]
    _id: AssetKey,
    #[icon(path = "icons/toggle-right.icon", opt = FeatherIcon::ToggleRight)]
    _id: AssetKey,
    #[icon(path = "icons/github.icon", opt = FeatherIcon::Github)]
    _id: AssetKey,
    #[icon(path = "icons/align-center.icon", opt = FeatherIcon::AlignCenter)]
    _id: AssetKey,
    #[icon(path = "icons/edit-2.icon", opt = FeatherIcon::EditTwo)]
    _id: AssetKey,
    #[icon(path = "icons/bell.icon", opt = FeatherIcon::Bell)]
    _id: AssetKey,
    #[icon(path = "icons/alert-octagon.icon", opt = FeatherIcon::AlertOctagon)]
    _id: AssetKey,
    #[icon(path = "icons/airplay.icon", opt = FeatherIcon::Airplay)]
    _id: AssetKey,
    #[icon(path = "icons/umbrella.icon", opt = FeatherIcon::Umbrella)]
    _id: AssetKey,
    #[icon(path = "icons/battery-charging.icon", opt = FeatherIcon::BatteryCharging)]
    _id: AssetKey,
    #[icon(path = "icons/phone.icon", opt = FeatherIcon::Phone)]
    _id: AssetKey,
    #[icon(path = "icons/gift.icon", opt = FeatherIcon::Gift)]
    _id: AssetKey,
    #[icon(path = "icons/info.icon", opt = FeatherIcon::Info)]
    _id: AssetKey,
    #[icon(path = "icons/repeat.icon", opt = FeatherIcon::Repeat)]
    _id: AssetKey,
    #[icon(path = "icons/twitter.icon", opt = FeatherIcon::Twitter)]
    _id: AssetKey,
    #[icon(path = "icons/cpu.icon", opt = FeatherIcon::Cpu)]
    _id: AssetKey,
    #[icon(path = "icons/type.icon", opt = FeatherIcon::Type)]
    _id: AssetKey,
    #[icon(path = "icons/codepen.icon", opt = FeatherIcon::Codepen)]
    _id: AssetKey,
    #[icon(path = "icons/arrow-up-circle.icon", opt = FeatherIcon::ArrowUpCircle)]
    _id: AssetKey,
    #[icon(path = "icons/battery.icon", opt = FeatherIcon::Battery)]
    _id: AssetKey,
    #[icon(path = "icons/figma.icon", opt = FeatherIcon::Figma)]
    _id: AssetKey,
    #[icon(path = "icons/sun.icon", opt = FeatherIcon::Sun)]
    _id: AssetKey,
    #[icon(path = "icons/trash-2.icon", opt = FeatherIcon::TrashTwo)]
    _id: AssetKey,
    #[icon(path = "icons/volume-1.icon", opt = FeatherIcon::VolumeOne)]
    _id: AssetKey,
    #[icon(path = "icons/map.icon", opt = FeatherIcon::Map)]
    _id: AssetKey,
    #[icon(path = "icons/cloud-rain.icon", opt = FeatherIcon::CloudRain)]
    _id: AssetKey,
    #[icon(path = "icons/target.icon", opt = FeatherIcon::Target)]
    _id: AssetKey,
    #[icon(path = "icons/chevrons-right.icon", opt = FeatherIcon::ChevronsRight)]
    _id: AssetKey,
    #[icon(path = "icons/skip-back.icon", opt = FeatherIcon::SkipBack)]
    _id: AssetKey,
    #[icon(path = "icons/folder.icon", opt = FeatherIcon::Folder)]
    _id: AssetKey,
    #[icon(path = "icons/hash.icon", opt = FeatherIcon::Hash)]
    _id: AssetKey,
    #[icon(path = "icons/phone-off.icon", opt = FeatherIcon::PhoneOff)]
    _id: AssetKey,
    #[icon(path = "icons/frown.icon", opt = FeatherIcon::Frown)]
    _id: AssetKey,
    #[icon(path = "icons/arrow-right-circle.icon", opt = FeatherIcon::ArrowRightCircle)]
    _id: AssetKey,
    #[icon(path = "icons/circle.icon", opt = FeatherIcon::Circle)]
    _id: AssetKey,
    #[icon(path = "icons/book-open.icon", opt = FeatherIcon::BookOpen)]
    _id: AssetKey,
    #[icon(path = "icons/image.icon", opt = FeatherIcon::Image)]
    _id: AssetKey,
    #[icon(path = "icons/refresh-ccw.icon", opt = FeatherIcon::RefreshCCW)]
    _id: AssetKey,
    #[icon(path = "icons/at-sign.icon", opt = FeatherIcon::AtSign)]
    _id: AssetKey,
    #[icon(path = "icons/zoom-in.icon", opt = FeatherIcon::ZoomIn)]
    _id: AssetKey,
    #[icon(path = "icons/user-minus.icon", opt = FeatherIcon::UserMinus)]
    _id: AssetKey,
    #[icon(path = "icons/chevrons-up.icon", opt = FeatherIcon::ChevronsUp)]
    _id: AssetKey,
    #[icon(path = "icons/corner-left-down.icon", opt = FeatherIcon::CornerLeftDown)]
    _id: AssetKey,
    #[icon(path = "icons/send.icon", opt = FeatherIcon::Send)]
    _id: AssetKey,
    #[icon(path = "icons/arrow-right.icon", opt = FeatherIcon::ArrowRight)]
    _id: AssetKey,
    #[icon(path = "icons/git-merge.icon", opt = FeatherIcon::GitMerge)]
    _id: AssetKey,
    #[icon(path = "icons/save.icon", opt = FeatherIcon::Save)]
    _id: AssetKey,
    #[icon(path = "icons/phone-outgoing.icon", opt = FeatherIcon::PhoneOutgoing)]
    _id: AssetKey,
    #[icon(path = "icons/cloud.icon", opt = FeatherIcon::Cloud)]
    _id: AssetKey,
    #[icon(path = "icons/arrow-left-circle.icon", opt = FeatherIcon::ArrowLeftCircle)]
    _id: AssetKey,
    #[icon(path = "icons/scissors.icon", opt = FeatherIcon::Scissors)]
    _id: AssetKey,
    #[icon(path = "icons/corner-left-up.icon", opt = FeatherIcon::CornerLeftUp)]
    _id: AssetKey,
    #[icon(path = "icons/minus-square.icon", opt = FeatherIcon::MinusSquare)]
    _id: AssetKey,
    #[icon(path = "icons/upload-cloud.icon", opt = FeatherIcon::UploadCloud)]
    _id: AssetKey,
    #[icon(path = "icons/columns.icon", opt = FeatherIcon::Columns)]
    _id: AssetKey,
    #[icon(path = "icons/package.icon", opt = FeatherIcon::Package)]
    _id: AssetKey,
    #[icon(path = "icons/rewind.icon", opt = FeatherIcon::Rewind)]
    _id: AssetKey,
    #[icon(path = "icons/maximize.icon", opt = FeatherIcon::Maximize)]
    _id: AssetKey,
    #[icon(path = "icons/volume-x.icon", opt = FeatherIcon::VolumeX)]
    _id: AssetKey,
    #[icon(path = "icons/share-2.icon", opt = FeatherIcon::ShareTwo)]
    _id: AssetKey,
    #[icon(path = "icons/shield.icon", opt = FeatherIcon::Shield)]
    _id: AssetKey,
    #[icon(path = "icons/wind.icon", opt = FeatherIcon::Wind)]
    _id: AssetKey,
    #[icon(path = "icons/life-buoy.icon", opt = FeatherIcon::LifeBuoy)]
    _id: AssetKey,
    #[icon(path = "icons/zap.icon", opt = FeatherIcon::Zap)]
    _id: AssetKey,
    #[icon(path = "icons/more-vertical.icon", opt = FeatherIcon::MoreVertical)]
    _id: AssetKey,
    #[icon(path = "icons/users.icon", opt = FeatherIcon::Users)]
    _id: AssetKey,
    #[icon(path = "icons/twitch.icon", opt = FeatherIcon::Twitch)]
    _id: AssetKey,
    #[icon(path = "icons/refresh-cw.icon", opt = FeatherIcon::RefreshCW)]
    _id: AssetKey,
    #[icon(path = "icons/toggle-left.icon", opt = FeatherIcon::ToggleLeft)]
    _id: AssetKey,
    #[icon(path = "icons/plus-square.icon", opt = FeatherIcon::PlusSquare)]
    _id: AssetKey,
    #[icon(path = "icons/octagon.icon", opt = FeatherIcon::Octagon)]
    _id: AssetKey,
    #[icon(path = "icons/heart.icon", opt = FeatherIcon::Heart)]
    _id: AssetKey,
    #[icon(path = "icons/video.icon", opt = FeatherIcon::Video)]
    _id: AssetKey,
    #[icon(path = "icons/arrow-down-circle.icon", opt = FeatherIcon::ArrowDownCircle)]
    _id: AssetKey,
    #[icon(path = "icons/arrow-down-left.icon", opt = FeatherIcon::ArrowDownLeft)]
    _id: AssetKey,
    #[icon(path = "icons/trello.icon", opt = FeatherIcon::Trello)]
    _id: AssetKey,
    #[icon(path = "icons/home.icon", opt = FeatherIcon::Home)]
    _id: AssetKey,
    #[icon(path = "icons/pause.icon", opt = FeatherIcon::Pause)]
    _id: AssetKey,
    #[icon(path = "icons/align-right.icon", opt = FeatherIcon::AlignRight)]
    _id: AssetKey,
    #[icon(path = "icons/table.icon", opt = FeatherIcon::Table)]
    _id: AssetKey,
    #[icon(path = "icons/corner-up-right.icon", opt = FeatherIcon::CornerUpRight)]
    _id: AssetKey,
    #[icon(path = "icons/corner-up-left.icon", opt = FeatherIcon::CornerUpLeft)]
    _id: AssetKey,
    #[icon(path = "icons/divide-square.icon", opt = FeatherIcon::DivideSquare)]
    _id: AssetKey,
    #[icon(path = "icons/mic-off.icon", opt = FeatherIcon::MicOff)]
    _id: AssetKey,
    #[icon(path = "icons/sunset.icon", opt = FeatherIcon::Sunset)]
    _id: AssetKey,
    #[icon(path = "icons/compass.icon", opt = FeatherIcon::Compass)]
    _id: AssetKey,
    #[icon(path = "icons/folder-minus.icon", opt = FeatherIcon::FolderMinus)]
    _id: AssetKey,
    #[icon(path = "icons/alert-circle.icon", opt = FeatherIcon::AlertCircle)]
    _id: AssetKey,
    #[icon(path = "icons/video-off.icon", opt = FeatherIcon::VideoOff)]
    _id: AssetKey,
    #[icon(path = "icons/power.icon", opt = FeatherIcon::Power)]
    _id: AssetKey,
    #[icon(path = "icons/crop.icon", opt = FeatherIcon::Crop)]
    _id: AssetKey,
    #[icon(path = "icons/mail.icon", opt = FeatherIcon::Mail)]
    _id: AssetKey,
    #[icon(path = "icons/share.icon", opt = FeatherIcon::Share)]
    _id: AssetKey,
    #[icon(path = "icons/underline.icon", opt = FeatherIcon::Underline)]
    _id: AssetKey,
    #[icon(path = "icons/credit-card.icon", opt = FeatherIcon::CreditCard)]
    _id: AssetKey,
    #[icon(path = "icons/cast.icon", opt = FeatherIcon::Cast)]
    _id: AssetKey,
    #[icon(path = "icons/x-octagon.icon", opt = FeatherIcon::XOctagon)]
    _id: AssetKey,
    #[icon(path = "icons/log-out.icon", opt = FeatherIcon::LogOut)]
    _id: AssetKey,
    #[icon(path = "icons/sidebar.icon", opt = FeatherIcon::Sidebar)]
    _id: AssetKey,
    #[icon(path = "icons/align-left.icon", opt = FeatherIcon::AlignLeft)]
    _id: AssetKey,
    #[icon(path = "icons/chevron-down.icon", opt = FeatherIcon::ChevronDown)]
    _id: AssetKey,
    #[icon(path = "icons/chevron-up.icon", opt = FeatherIcon::ChevronUp)]
    _id: AssetKey,
    #[icon(path = "icons/bar-chart.icon", opt = FeatherIcon::BarChart)]
    _id: AssetKey,
    #[icon(path = "icons/inbox.icon", opt = FeatherIcon::Inbox)]
    _id: AssetKey,
    #[icon(path = "icons/pen-tool.icon", opt = FeatherIcon::PenTool)]
    _id: AssetKey,
    #[icon(path = "icons/camera.icon", opt = FeatherIcon::Camera)]
    _id: AssetKey,
    #[icon(path = "icons/eye-off.icon", opt = FeatherIcon::EyeOff)]
    _id: AssetKey,
    #[icon(path = "icons/sliders.icon", opt = FeatherIcon::Sliders)]
    _id: AssetKey,
    #[icon(path = "icons/pocket.icon", opt = FeatherIcon::Pocket)]
    _id: AssetKey,
    #[icon(path = "icons/upload.icon", opt = FeatherIcon::Upload)]
    _id: AssetKey,
    #[icon(path = "icons/thumbs-down.icon", opt = FeatherIcon::ThumbsDown)]
    _id: AssetKey,
    #[icon(path = "icons/chrome.icon", opt = FeatherIcon::Chrome)]
    _id: AssetKey,
    #[icon(path = "icons/zap-off.icon", opt = FeatherIcon::ZapOff)]
    _id: AssetKey,
    #[icon(path = "icons/check-square.icon", opt = FeatherIcon::CheckSquare)]
    _id: AssetKey,
    #[icon(path = "icons/file-text.icon", opt = FeatherIcon::FileText)]
    _id: AssetKey,
    #[icon(path = "icons/wifi.icon", opt = FeatherIcon::Wifi)]
    _id: AssetKey,
    #[icon(path = "icons/chevrons-down.icon", opt = FeatherIcon::ChevronsDown)]
    _id: AssetKey,
    #[icon(path = "icons/folder-plus.icon", opt = FeatherIcon::FolderPlus)]
    _id: AssetKey,
    #[icon(path = "icons/arrow-left.icon", opt = FeatherIcon::ArrowLeft)]
    _id: AssetKey,
    #[icon(path = "icons/instagram.icon", opt = FeatherIcon::Instagram)]
    _id: AssetKey,
    #[icon(path = "icons/mouse-pointer.icon", opt = FeatherIcon::MousePointer)]
    _id: AssetKey,
    #[icon(path = "icons/slack.icon", opt = FeatherIcon::Slack)]
    _id: AssetKey,
    #[icon(path = "icons/file-plus.icon", opt = FeatherIcon::FilePlus)]
    _id: AssetKey,
    #[icon(path = "icons/chevrons-left.icon", opt = FeatherIcon::ChevronsLeft)]
    _id: AssetKey,
    #[icon(path = "icons/move.icon", opt = FeatherIcon::Move)]
    _id: AssetKey,
    #[icon(path = "icons/align-justify.icon", opt = FeatherIcon::AlignJustify)]
    _id: AssetKey,
    #[icon(path = "icons/book.icon", opt = FeatherIcon::Book)]
    _id: AssetKey,
    #[icon(path = "icons/phone-call.icon", opt = FeatherIcon::PhoneCall)]
    _id: AssetKey,
    #[icon(path = "icons/bar-chart-2.icon", opt = FeatherIcon::BarChart2)]
    _id: AssetKey,
    #[icon(path = "icons/slash.icon", opt = FeatherIcon::Slash)]
    _id: AssetKey,
    #[icon(path = "icons/shuffle.icon", opt = FeatherIcon::Shuffle)]
    _id: AssetKey,
    #[icon(path = "icons/facebook.icon", opt = FeatherIcon::Facebook)]
    _id: AssetKey,
    #[icon(path = "icons/list.icon", opt = FeatherIcon::List)]
    _id: AssetKey,
    #[icon(path = "icons/minimize.icon", opt = FeatherIcon::Minimize)]
    _id: AssetKey,
    #[icon(path = "icons/tool.icon", opt = FeatherIcon::Tool)]
    _id: AssetKey,
    #[icon(path = "icons/coffee.icon", opt = FeatherIcon::Coffee)]
    _id: AssetKey,
    #[icon(path = "icons/unlock.icon", opt = FeatherIcon::Unlock)]
    _id: AssetKey,
    #[icon(path = "icons/moon.icon", opt = FeatherIcon::Moon)]
    _id: AssetKey,
    #[icon(path = "icons/phone-incoming.icon", opt = FeatherIcon::PhoneIncoming)]
    _id: AssetKey,
    #[icon(path = "icons/message-square.icon", opt = FeatherIcon::MessageSquare)]
    _id: AssetKey,
    #[icon(path = "icons/edit-3.icon", opt = FeatherIcon::EditThree)]
    _id: AssetKey,
    #[icon(path = "icons/external-link.icon", opt = FeatherIcon::ExternalLink)]
    _id: AssetKey,
    #[icon(path = "icons/maximize-2.icon", opt = FeatherIcon::MaximizeTwo)]
    _id: AssetKey,
    #[icon(path = "icons/thumbs-up.icon", opt = FeatherIcon::ThumbsUp)]
    _id: AssetKey,
    #[icon(path = "icons/alert-triangle.icon", opt = FeatherIcon::AlertTriangle)]
    _id: AssetKey,
    #[icon(path = "icons/percent.icon", opt = FeatherIcon::Percent)]
    _id: AssetKey,
    #[icon(path = "icons/framer.icon", opt = FeatherIcon::Framer)]
    _id: AssetKey,
    #[icon(path = "icons/lock.icon", opt = FeatherIcon::Lock)]
    _id: AssetKey,
    #[icon(path = "icons/wifi-off.icon", opt = FeatherIcon::WifiOff)]
    _id: AssetKey,
    #[icon(path = "icons/pie-chart.icon", opt = FeatherIcon::PieChart)]
    _id: AssetKey,
    #[icon(path = "icons/trending-down.icon", opt = FeatherIcon::TrendingDown)]
    _id: AssetKey,
    #[icon(path = "icons/git-branch.icon", opt = FeatherIcon::GitBranch)]
    _id: AssetKey,
    #[icon(path = "icons/x-square.icon", opt = FeatherIcon::XSquare)]
    _id: AssetKey,
    #[icon(path = "icons/menu.icon", opt = FeatherIcon::Menu)]
    _id: AssetKey,
    #[icon(path = "icons/bookmark.icon", opt = FeatherIcon::Bookmark)]
    _id: AssetKey,
    #[icon(path = "icons/square.icon", opt = FeatherIcon::Square)]
    _id: AssetKey,
    #[icon(path = "icons/divide.icon", opt = FeatherIcon::Divide)]
    _id: AssetKey,
    #[icon(path = "icons/shopping-cart.icon", opt = FeatherIcon::ShoppingCart)]
    _id: AssetKey,
    #[icon(path = "icons/phone-forwarded.icon", opt = FeatherIcon::PhoneForwarded)]
    _id: AssetKey,
    #[icon(path = "icons/box.icon", opt = FeatherIcon::Box)]
    _id: AssetKey,
    #[icon(path = "icons/dribbble.icon", opt = FeatherIcon::Dribble)]
    _id: AssetKey,
    #[icon(path = "icons/check.icon", opt = FeatherIcon::Check)]
    _id: AssetKey,
    #[icon(path = "icons/trash.icon", opt = FeatherIcon::Trash)]
    _id: AssetKey,
    #[icon(path = "icons/smartphone.icon", opt = FeatherIcon::Smartphone)]
    _id: AssetKey,
    #[icon(path = "icons/activity.icon", opt = FeatherIcon::Activity)]
    _id: AssetKey,
    #[icon(path = "icons/help-circle.icon", opt = FeatherIcon::HelpCircle)]
    _id: AssetKey,
    #[icon(path = "icons/database.icon", opt = FeatherIcon::Database)]
    _id: AssetKey,
    #[icon(path = "icons/user.icon", opt = FeatherIcon::User)]
    _id: AssetKey,
    #[icon(path = "icons/minimize-2.icon", opt = FeatherIcon::MinimizeTwo)]
    _id: AssetKey,
    #[icon(path = "icons/bell-off.icon", opt = FeatherIcon::BellOff)]
    _id: AssetKey,
    #[icon(path = "icons/arrow-down.icon", opt = FeatherIcon::ArrowDown)]
    _id: AssetKey,
    #[icon(path = "icons/volume-2.icon", opt = FeatherIcon::VolumeTwo)]
    _id: AssetKey,
    #[icon(path = "icons/sunrise.icon", opt = FeatherIcon::Sunrise)]
    _id: AssetKey,
    #[icon(path = "icons/award.icon", opt = FeatherIcon::Award)]
    _id: AssetKey,
    #[icon(path = "icons/pause-circle.icon", opt = FeatherIcon::PauseCircle)]
    _id: AssetKey,
    #[icon(path = "icons/radio.icon", opt = FeatherIcon::Radio)]
    _id: AssetKey,
    #[icon(path = "icons/user-x.icon", opt = FeatherIcon::UserX)]
    _id: AssetKey,
    #[icon(path = "icons/headphones.icon", opt = FeatherIcon::Headphones)]
    _id: AssetKey,
    #[icon(path = "icons/zoom-out.icon", opt = FeatherIcon::ZoomOut)]
    _id: AssetKey,
    #[icon(path = "icons/disc.icon", opt = FeatherIcon::Disc)]
    _id: AssetKey,
    #[icon(path = "icons/clipboard.icon", opt = FeatherIcon::Clipboard)]
    _id: AssetKey,
    #[icon(path = "icons/user-check.icon", opt = FeatherIcon::UserCheck)]
    _id: AssetKey,
    #[icon(path = "icons/cloud-drizzle.icon", opt = FeatherIcon::CloudDrizzle)]
    _id: AssetKey,
    #[icon(path = "icons/speaker.icon", opt = FeatherIcon::Speaker)]
    _id: AssetKey,
    #[icon(path = "icons/calendar.icon", opt = FeatherIcon::Calendar)]
    _id: AssetKey,
    #[icon(path = "icons/crosshair.icon", opt = FeatherIcon::Crosshair)]
    _id: AssetKey,
    #[icon(path = "icons/more-horizontal.icon", opt = FeatherIcon::MoreHorizontal)]
    _id: AssetKey,
    #[icon(path = "icons/dollar-sign.icon", opt = FeatherIcon::DollarSign)]
    _id: AssetKey,
    #[icon(path = "icons/watch.icon", opt = FeatherIcon::Watch)]
    _id: AssetKey,
    #[icon(path = "icons/key.icon", opt = FeatherIcon::Key)]
    _id: AssetKey,
    #[icon(path = "icons/delete.icon", opt = FeatherIcon::Delete)]
    _id: AssetKey,
    #[icon(path = "icons/anchor.icon", opt = FeatherIcon::Anchor)]
    _id: AssetKey,
    #[icon(path = "icons/plus.icon", opt = FeatherIcon::Plus)]
    _id: AssetKey,
    #[icon(path = "icons/star.icon", opt = FeatherIcon::Star)]
    _id: AssetKey,
    #[icon(path = "icons/linkedin.icon", opt = FeatherIcon::LinkedIn)]
    _id: AssetKey,
    #[icon(path = "icons/eye.icon", opt = FeatherIcon::Eye)]
    _id: AssetKey,
    #[icon(path = "icons/clock.icon", opt = FeatherIcon::Clock)]
    _id: AssetKey,
    #[icon(path = "icons/tv.icon", opt = FeatherIcon::TV)]
    _id: AssetKey,
    #[icon(path = "icons/flag.icon", opt = FeatherIcon::Flag)]
    _id: AssetKey,
    #[icon(path = "icons/copy.icon", opt = FeatherIcon::Copy)]
    _id: AssetKey,
    #[icon(path = "icons/loader.icon", opt = FeatherIcon::Loader)]
    _id: AssetKey,
    #[icon(path = "icons/log-in.icon", opt = FeatherIcon::LogIn)]
    _id: AssetKey,
    #[icon(path = "icons/film.icon", opt = FeatherIcon::Film)]
    _id: AssetKey,
    #[icon(path = "icons/cloud-off.icon", opt = FeatherIcon::CloudOff)]
    _id: AssetKey,
    #[icon(path = "icons/play-circle.icon", opt = FeatherIcon::PlayCircle)]
    _id: AssetKey,
    #[icon(path = "icons/truck.icon", opt = FeatherIcon::Truck)]
    _id: AssetKey,
    #[icon(path = "icons/thermometer.icon", opt = FeatherIcon::Thermometer)]
    _id: AssetKey,
    #[icon(path = "icons/minus.icon", opt = FeatherIcon::Minus)]
    _id: AssetKey,
    #[icon(path = "icons/rss.icon", opt = FeatherIcon::RSS)]
    _id: AssetKey,
    #[icon(path = "icons/filter.icon", opt = FeatherIcon::Filter)]
    _id: AssetKey,
    #[icon(path = "icons/bluetooth.icon", opt = FeatherIcon::Bluetooth)]
    _id: AssetKey,
    #[icon(path = "icons/stop-circle.icon", opt = FeatherIcon::StopCircle)]
    _id: AssetKey,
    #[icon(path = "icons/user-plus.icon", opt = FeatherIcon::UserPlus)]
    _id: AssetKey,
    #[icon(path = "icons/rotate-ccw.icon", opt = FeatherIcon::RotateCCW)]
    _id: AssetKey,
    #[icon(path = "icons/gitlab.icon", opt = FeatherIcon::Gitlab)]
    _id: AssetKey,
    #[icon(path = "icons/monitor.icon", opt = FeatherIcon::Monitor)]
    _id: AssetKey,
    #[icon(path = "icons/briefcase.icon", opt = FeatherIcon::Briefcase)]
    _id: AssetKey,
    #[icon(path = "icons/minus-circle.icon", opt = FeatherIcon::MinusCircle)]
    _id: AssetKey,
    #[icon(path = "icons/arrow-up.icon", opt = FeatherIcon::ArrowUp)]
    _id: AssetKey,
    #[icon(path = "icons/cloud-lightning.icon", opt = FeatherIcon::CloudLightning)]
    _id: AssetKey,
    #[icon(path = "icons/youtube.icon", opt = FeatherIcon::Youtube)]
    _id: AssetKey,
    #[icon(path = "icons/download-cloud.icon", opt = FeatherIcon::DownloadCloud)]
    _id: AssetKey,
    #[icon(path = "icons/navigation.icon", opt = FeatherIcon::Navigation)]
    _id: AssetKey,
    #[icon(path = "icons/link-2.icon", opt = FeatherIcon::LinkTwo)]
    _id: AssetKey,
    #[icon(path = "icons/music.icon", opt = FeatherIcon::Music)]
    _id: AssetKey,
    #[icon(path = "icons/grid.icon", opt = FeatherIcon::Grid)]
    _id: AssetKey,
    #[icon(path = "icons/arrow-up-right.icon", opt = FeatherIcon::ArrowUpRight)]
    _id: AssetKey,
    #[icon(path = "icons/hexagon.icon", opt = FeatherIcon::Hexagon)]
    _id: AssetKey,
    #[icon(path = "icons/corner-down-right.icon", opt = FeatherIcon::CornerDownRight)]
    _id: AssetKey,
    #[icon(path = "icons/voicemail.icon", opt = FeatherIcon::Voicemail)]
    _id: AssetKey,
    #[icon(path = "icons/meh.icon", opt = FeatherIcon::Meh)]
    _id: AssetKey,
    #[icon(path = "icons/hard-drive.icon", opt = FeatherIcon::HardDrive)]
    _id: AssetKey,
    #[icon(path = "icons/chevron-left.icon", opt = FeatherIcon::ChevronLeft)]
    _id: AssetKey,
    #[icon(path = "icons/server.icon", opt = FeatherIcon::Server)]
    _id: AssetKey,
    #[icon(path = "icons/fast-forward.icon", opt = FeatherIcon::FastForward)]
    _id: AssetKey,
    #[icon(path = "icons/download.icon", opt = FeatherIcon::Download)]
    _id: AssetKey,
    #[bytes(path = "something.dat", group = generated)]
    _id: AssetKey,
}
