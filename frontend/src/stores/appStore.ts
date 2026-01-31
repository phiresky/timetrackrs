import { makeAutoObservable } from 'mobx'
import type { ProgressReport } from '../server'
import { progressEvents } from '../lib/api'

class AppStore {
  progressReports: ProgressReport[] = []
  private eventSource: EventSource | null = null

  constructor() {
    makeAutoObservable(this)
  }

  startProgressListener() {
    if (this.eventSource) return

    this.eventSource = progressEvents((reports) => {
      this.setProgressReports(reports)
    })
  }

  stopProgressListener() {
    if (this.eventSource) {
      this.eventSource.close()
      this.eventSource = null
    }
  }

  setProgressReports(reports: ProgressReport[]) {
    this.progressReports = reports
  }

  get activeProgressReports() {
    return this.progressReports.filter((r) => !r.done)
  }
}

export const appStore = new AppStore()
