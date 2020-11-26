import React from "react"
import { Activity } from "../api"
import { ExtractedInfo } from "../server"

type _UseSoftware<T> = T extends { type: "InteractWithDevice" } ? T : never
export type UseSoftware = _UseSoftware<ExtractedInfo>
type Keyed<
	T extends { [k in discriminator]: string | number | symbol },
	discriminator extends keyof T
> = {
	[k in T[discriminator]]: Omit<
		Extract<T, { [z in discriminator]: k }>,
		discriminator
	>
}
export type KeyedExtractedInfo = Keyed<ExtractedInfo, "type">

type KeyedUseSpecificSoftware = Keyed<UseSoftware["specific"], "type">

export type KeyedOuterUseSpecificSoftware = {
	[k in keyof KeyedUseSpecificSoftware]: UseSoftware & {
		specific: KeyedUseSpecificSoftware[k]
	}
}

type KeyedReactComp<T> = { [k in keyof T]: React.ComponentType<T[k]> }

const useSpecificSoftwareComponents: KeyedReactComp<KeyedOuterUseSpecificSoftware> = {
	Shell(e) {
		return <span>Shell in {e.specific.cwd}</span>
	},
	WebBrowser(e) {
		return <span>Browser at {e.specific.domain}</span>
	},
	SoftwareDevelopment(e) {
		return <span>Software Development of {e.specific.project_path}</span>
	},
	MediaPlayer(e) {
		return <span>Consumed Media: {e.specific.media_name}</span>
	},
	DeviceStateChange(e) {
		return (
			<span>
				{e.specific.change} device {e.general.hostname}
			</span>
		)
	},
	Unknown(e) {
		return (
			<span>
				Used {e.general.device_type}: {e.general.title}
			</span>
		)
	},
}

/*const softwareComponents: {k in keyof }*/
export const entryComponents: KeyedReactComp<KeyedExtractedInfo> = {
	PhysicalActivity(e) {
		return <div>*dance*</div>
	},
	InteractWithDevice(e) {
		const Comp = useSpecificSoftwareComponents[e.specific.type]
		return <Comp {...(e as any)} />
	},
}

export class Entry extends React.Component<Activity> {
	render(): React.ReactNode {
		const { data } = this.props
		const E = (entryComponents[
			data.info.type
		] as unknown) as React.ComponentType<ExtractedInfo>
		console.log(E)
		return <E {...data.info} />
		//return "unk: " + data.software?.title
	}
}
