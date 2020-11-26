type DateTime<T> = string
type Local = unknown
type Timestamptz = string
type Text10Choices = string
type Text100Choices = string
type Text1000Choices = string
type Text10000Choices = string
type Text100000Choices = string
export type Sampler =
	| { type: "RandomSampler"; avg_time: number }
	| { type: "Explicit"; duration: number }
export type EventData =
	| { data_type: "x11_v2"; data: X11EventData }
	| { data_type: "windows_v1"; data: WindowsEventData }
	| { data_type: "app_usage_v1"; data: AppUsageEntry }
	| { data_type: "journald"; data: JournaldEntry }
export type X11EventData = {
	os_info: OsInfo
	desktop_names: string[]
	current_desktop_id: number
	focused_window: number
	ms_since_user_input: number
	ms_until_screensaver: number
	screensaver_window: number
	windows: X11WindowData[]
}
export type X11WindowData = {
	window_id: number
	geometry: X11WindowGeometry
	process: ProcessData | null
	window_properties: { [key: string]: J }
}
export type X11WindowGeometry = {
	x: number
	y: number
	width: number
	height: number
}
export type ProcessData = {
	pid: number
	name: string
	cmd: string[]
	exe: string
	cwd: string
	memory_kB: number
	parent: number | null
	status: string
	start_time: DateTime<Utc>
	cpu_usage: number | null
}
export type OsInfo = {
	os_type: string
	version: string
	batteries: number | null
	hostname: string
	machine_id: string | null
}
export type ExtractedInfo =
	| {
			type: "InteractWithDevice"
			general: GeneralSoftware
			specific: SpecificSoftware
	  }
	| { type: "PhysicalActivity"; activity_type: Text100Choices }
export type EnrichedExtractedInfo = { tags: string[]; info: ExtractedInfo }
export enum SoftwareDeviceType {
	Desktop = "Desktop",
	Laptop = "Laptop",
	Smartphone = "Smartphone",
	Tablet = "Tablet",
}
// - some generic identifier that can be looked up elsewhere. i.e. something that should be unique within the corresponding scope of the surrounding object
export type Identifier = string
export enum DeviceStateChange {
	PowerOn = "PowerOn",
	PowerOff = "PowerOff",
	Sleep = "Sleep",
	Wakeup = "Wakeup",
}
export type GeneralSoftware = {
	hostname: Text100Choices
	device_type: SoftwareDeviceType
	device_os: Text10Choices
	title: Text10000Choices
	identifier: Identifier
	unique_name: Text100Choices
	opened_filepath: Text10000Choices | null
}
export type SpecificSoftware =
	| { type: "DeviceStateChange"; change: DeviceStateChange }
	| {
			type: "WebBrowser"
			url: Text10000Choices | null
			domain: Text1000Choices | null
	  }
	| {
			type: "Shell"
			cwd: Text1000Choices
			cmd: Text10000Choices
			zsh_histdb_session_id: Identifier
	  }
	| {
			type: "MediaPlayer"
			media_filename: Text1000Choices
			media_type: MediaType
			media_name: Text1000Choices
	  }
	| {
			type: "SoftwareDevelopment"
			project_path: Text100Choices | null
			file_path: Text1000Choices
	  }
	| { type: "Unknown" }
export enum MediaType {
	Audio = "Audio",
	Video = "Video",
}
