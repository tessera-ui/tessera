import java.io.File
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.logging.LogLevel
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.Optional
import org.gradle.api.tasks.TaskAction

open class BuildTask : DefaultTask() {
    @Input
    var rootDirRel: String? = null
    @Input
    var target: String? = null
    @Input
    var release: Boolean? = null
    @Input
    @Optional
    var profilingOutput: String? = null
    @Input
    var debugDirtyOverlay: Boolean = false

    @TaskAction
    fun build() {
        val rootDirRel = rootDirRel ?: throw GradleException("rootDirRel cannot be null")
        val target = target ?: throw GradleException("target cannot be null")
        val release = release ?: throw GradleException("release cannot be null")

        project.exec {
            workingDir(File(project.projectDir, rootDirRel))
            executable("cargo")
            args(listOf("tessera", "android", "rust-build"))
            if (project.logger.isEnabled(LogLevel.DEBUG)) {
                args("-vv")
            } else if (project.logger.isEnabled(LogLevel.INFO)) {
                args("-v")
            }
            if (release) {
                args("--release")
            }
            profilingOutput?.takeIf { it.isNotBlank() }?.let { outputPath ->
                args("--profiling-output", outputPath)
            }
            if (debugDirtyOverlay) {
                args("--debug-dirty-overlay")
            }
            args(target)
        }.assertNormalExitValue()
    }
}
