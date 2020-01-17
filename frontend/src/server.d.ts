type DateTime<T> = string
type Local = unknown
type Timestamptz = string
type Text10 = string
type Text100 = string
type Text1000 = string
type Text10000 = string
type Text100000 = string
export type Activity = {
	id: number
	timestamp: Timestamptz
	data_type: string
	data_type_version: number
	sampler: Sampler
	sampler_sequence_id: string
	data: string
}
export type Sampler = { type: "RandomSampler"; avg_time: number }
export type CapturedData = { data_type: "x11"; data: X11CapturedData }
export type X11CapturedData = {
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
	start_time: DateTime<Local>
	cpu_usage: number
}
export type OsInfo = {
	os_type: string
	version: string
	batteries: number
	hostname: string
}
export type ExtractedInfo = {
	event_id: string
	software_development: SoftwareDevelopment | null
	shell: Shell | null
	web_browser: WebBrowser | null
	media_player: MediaPlayer | null
	software: Software | null
	physical_activity: PhysicalActivity | null
}
export type SoftwareDevelopment = {
	project_path: Text100 | null
	file_path: Text1000
}
export type Shell = {
	cwd: Text1000
	cmd: Text10000
	zsh_histdb_session_id: Identifier
}
export type WebBrowser = { url: Text10000; origin: Text1000; service: Text1000 }
export type MediaPlayer = {
	media_filename: Text1000
	media_type: MediaType
	media_name: Text1000
}
export type Software = {
	hostname: Text100
	device_type: SoftwareDeviceType
	device_os: Text10
	title: Text10000
	identifier: Identifier
	unique_name: Text100
}
export type PhysicalActivity = { activity_type: Text100 }
export enum MediaType {
	Audio = "Audio",
	Video = "Video",
}
export enum SoftwareDeviceType {
	Desktop = "Desktop",
	Laptop = "Laptop",
	Smartphone = "Smartphone",
	Tablet = "Tablet",
}
export type Identifier = string
