type DateTime<T> = string
type Local = unknown
type Timestamptz = string

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
	| {
			type: "PhysicalActivity"
			activity_type: string
	  }
export enum SoftwareDeviceType {
	Desktop = "Desktop",
	Laptop = "Laptop",
	Smartphone = "Smartphone",
	Tablet = "Tablet",
}

export type SpecificSoftware =
	| { type: "DeviceStateChange"; change: DeviceStateChange }
	| {
			type: "WebBrowser"
			url: string | null
			origin: string | null
			service: string | null
	  }
	| {
			type: "Shell"
			cwd: string
			cmd: string
			zsh_histdb_session_id: string
	  }
	| {
			type: "MediaPlayer"
			media_filename: string
			media_type: MediaType
			media_name: string
	  }
	| {
			type: "SoftwareDevelopment"
			project_path: string | null
			file_path: string
	  }
	| { type: "Unknown" }
export enum DeviceStateChange {
	PowerOn = "PowerOn",
	PowerOff = "PowerOff",
	Sleep = "Sleep",
	Wakeup = "Wakeup",
}
export type GeneralSoftware = {
	hostname: string
	device_type: SoftwareDeviceType
	device_os: string
	title: string
	identifier: string
	unique_name: string
	opened_filepath: string | null
}

export enum MediaType {
	Audio = "Audio",
	Video = "Video",
}
