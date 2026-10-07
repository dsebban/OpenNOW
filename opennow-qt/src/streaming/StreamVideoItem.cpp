#include "streaming/StreamVideoItem.h"

#include "input/platform/WaylandPointerCapture.h"
#include "input/platform/MacPointerCapture.h"
#include "streaming/NativeStreamRuntime.h"
#include "streaming/rendering/NativeStreamRenderCallback.h"
#include "streaming/rendering/StreamSwapStallWatchdog.h"

#include <QCursor>
#include <QDateTime>
#include <QFile>
#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QMetaObject>
#include <QQmlEngine>
#include <QQuickWindow>
#include <QScreen>
#include <QThread>

#include <algorithm>
#include <atomic>
#include <cmath>
#include <utility>

QPointer<NativeStreamRuntime> StreamVideoItem::s_nativeRuntime;

namespace {
std::atomic<quint64> nextLiveTelemetryObserverId{0};
}

StreamVideoItem::StreamVideoItem(QQuickItem *parent)
    : StreamVideoItem(std::make_unique<MacPointerCapture>(), MacPointerCapture::isSupported(), parent)
{
}

StreamVideoItem::StreamVideoItem(std::unique_ptr<MacPointerCapture> pointerCapture,
                               bool usesMacPointerCapture, QQuickItem *parent)
    : QQuickItem(parent), m_waylandPointer(std::make_unique<WaylandPointerCapture>()),
      m_macPointer(std::move(pointerCapture)), m_usesMacPointerCapture(usesMacPointerCapture)
{
    connect(m_macPointer.get(), &MacPointerCapture::stateChanged, this, [this] {
        syncCaptureState();
        emit inputCaptureErrorChanged();
    }, Qt::QueuedConnection);
    connect(m_macPointer.get(), &MacPointerCapture::relativeMotion, this,
            [this](qint16 x, qint16 y) {
        if (m_captureActive && m_inputEnabled && m_relativeMouse && m_macPointer->locked() && s_nativeRuntime)
            s_nativeRuntime->submitMouseRelative(x, y);
    });
    connect(m_waylandPointer.get(), &WaylandPointerCapture::stateChanged, this, [this] {
        syncCaptureState();
        emit inputCaptureErrorChanged();
    }, Qt::QueuedConnection);
    connect(m_waylandPointer.get(), &WaylandPointerCapture::relativeMotion, this,
            [this](qint16 x, qint16 y) {
        if (m_captureActive && m_inputEnabled && m_relativeMouse && s_nativeRuntime)
            s_nativeRuntime->submitMouseRelative(x, y);
    });
    setFlag(ItemHasContents, true);
    setActiveFocusOnTab(true);
    setAcceptedMouseButtons(Qt::AllButtons);
    setKeepMouseGrab(true);
    setAcceptHoverEvents(true);
    m_frameStatsTimer.setInterval(1000);
    connect(&m_frameStatsTimer, &QTimer::timeout,
            this, &StreamVideoItem::frameGenerationStatsChanged);
    m_swapStatsTimer.setInterval(1000);
    connect(&m_swapStatsTimer, &QTimer::timeout, this, &StreamVideoItem::swapStatsChanged);
    m_liveTelemetryPath = qEnvironmentVariable("OPENNOW_LIVE_TELEMETRY");
    m_liveTelemetryObserverId = nextLiveTelemetryObserverId.fetch_add(1, std::memory_order_relaxed) + 1;
    if (!m_liveTelemetryPath.isEmpty())
        connect(&m_swapStatsTimer, &QTimer::timeout, this, &StreamVideoItem::writeLiveTelemetry);
    connect(this, &QQuickItem::visibleChanged, this, &StreamVideoItem::updateSwapGate,
            Qt::UniqueConnection);
    if (s_nativeRuntime) {
        m_serverCursorComposited = s_nativeRuntime->serverCursorComposited();
        connect(s_nativeRuntime, &NativeStreamRuntime::cursorCaptureChanged, this,
                [this](bool composited) {
            m_serverCursorComposited = composited;
            updateLocalCursor();
        });
        connect(s_nativeRuntime, &NativeStreamRuntime::cursorStateReset,
                this, &StreamVideoItem::resetRemoteCursor);
        connect(s_nativeRuntime, &NativeStreamRuntime::inputAllowedChanged,
                this, &StreamVideoItem::syncCaptureState);
        connect(s_nativeRuntime, &NativeStreamRuntime::inputCaptureReset, this, [this] {
            m_manualRelativeMouse.reset();
            releaseInput();
            m_rawInputActive = false;
            if (std::exchange(m_captureActive, false)) emit captureActiveChanged();
        });
        setRenderCallback(createNativeStreamRenderCallback(s_nativeRuntime));
        connect(s_nativeRuntime, &NativeStreamRuntime::frameAvailable,
                this, &StreamVideoItem::requestFrame);
        connect(s_nativeRuntime, &NativeStreamRuntime::cursorUpdated,
                this, &StreamVideoItem::applyRemoteCursor);
        connect(s_nativeRuntime, &NativeStreamRuntime::runningChanged, this, [this] {
            if (!s_nativeRuntime || !s_nativeRuntime->running()) {
                m_manualRelativeMouse.reset();
                resetRemoteCursor();
            }
            syncCaptureState();
        });
    }
    const auto attachWindow = [this](QQuickWindow *currentWindow) {
        if (m_inputWindow) m_inputWindow->removeEventFilter(this);
        m_inputWindow = currentWindow;
        if (m_inputWindow) m_inputWindow->installEventFilter(this);
        connectFrameSwaps();
        if (currentWindow) {
            connect(currentWindow, &QWindow::activeChanged,
                    this, &StreamVideoItem::syncCaptureState, Qt::UniqueConnection);
            connect(currentWindow, &QWindow::visibilityChanged,
                    this, &StreamVideoItem::updateSwapGate, Qt::UniqueConnection);
            connect(currentWindow, &QWindow::xChanged,
                    this, &StreamVideoItem::updateCursorConfinement, Qt::UniqueConnection);
            connect(currentWindow, &QWindow::yChanged,
                    this, &StreamVideoItem::updateCursorConfinement, Qt::UniqueConnection);
            connect(currentWindow, &QWindow::widthChanged,
                    this, &StreamVideoItem::resynchronizeInput, Qt::UniqueConnection);
            connect(currentWindow, &QWindow::heightChanged,
                    this, &StreamVideoItem::resynchronizeInput, Qt::UniqueConnection);
            connect(currentWindow, &QWindow::screenChanged,
                    this, &StreamVideoItem::resynchronizeInput, Qt::UniqueConnection);
        }
        syncCaptureState();
        updateSwapGate();
    };
    connect(this, &QQuickItem::windowChanged, this, attachWindow);
    attachWindow(window());
}

StreamVideoItem::~StreamVideoItem()
{
    disconnect(this, nullptr, this, nullptr);
    disconnect(m_frameSwapConnection);
    disconnect(m_frameUpdateConnection);
    releaseInput();
    if (s_nativeRuntime && s_nativeRuntime->running()) {
        bool rawInput = false;
        s_nativeRuntime->setCaptureActive(false, false, 0, &rawInput);
    }
}

QSize StreamVideoItem::videoSize() const
{
    return m_videoSize;
}

void StreamVideoItem::setVideoSize(const QSize &size)
{
    const auto normalized = size.isValid() && !size.isEmpty() ? size : QSize{};
    if (m_videoSize == normalized) return;
    m_videoSize = normalized;
    emit videoSizeChanged();
    update();
}

bool StreamVideoItem::renderCallbackAvailable() const
{
    return static_cast<bool>(m_renderCallback);
}

bool StreamVideoItem::nativeRuntimeAvailable() const
{
    return s_nativeRuntime && s_nativeRuntime->running();
}

bool StreamVideoItem::inputEnabled() const
{
    return m_inputEnabled;
}

void StreamVideoItem::setInputEnabled(bool enabled)
{
    if (m_inputEnabled == enabled) return;
    m_inputEnabled = enabled;
    syncCaptureState();
    emit inputEnabledChanged();
}

bool StreamVideoItem::captureActive() const
{
    return m_captureActive;
}

bool StreamVideoItem::clipboardPaste() const
{
    return m_clipboardPaste;
}

void StreamVideoItem::setClipboardPaste(bool enabled)
{
    if (m_clipboardPaste == enabled) return;
    m_clipboardPaste = enabled;
    emit clipboardPasteChanged();
}

QString StreamVideoItem::keyboardLayout() const
{
    return m_keyboardLayout;
}

void StreamVideoItem::setKeyboardLayout(const QString &layout)
{
    if (m_keyboardLayout == layout) return;
    m_keyboardLayout = layout;
    m_keyboardMap = PhysicalKeyMap::layoutFor(layout.toStdString());
    emit keyboardLayoutChanged();
}

QString StreamVideoItem::inputCaptureError() const
{
    return m_usesMacPointerCapture ? m_macPointer->error() : m_waylandPointer->error();
}

bool StreamVideoItem::relativeMouse() const
{
    return m_relativeMouse;
}

QVariantMap StreamVideoItem::shortcutBindings() const
{
    return m_shortcutBindings;
}

void StreamVideoItem::setShortcutBindings(const QVariantMap &bindings)
{
    if (m_shortcutBindings == bindings) return;
    m_shortcutBindings = bindings;
    emit shortcutBindingsChanged();
}

std::shared_ptr<StreamVideoRenderCallback> StreamVideoItem::renderCallback() const
{
    return m_renderCallback;
}

void StreamVideoItem::setRenderCallback(std::shared_ptr<StreamVideoRenderCallback> callback)
{
    if (m_renderCallback == callback) return;
    const auto wasAvailable = renderCallbackAvailable();
    m_renderCallback = std::move(callback);
    if (m_renderCallback) m_swapStatsTimer.start();
    else m_swapStatsTimer.stop();
    connectFrameSwaps();
    syncSwapGate();
    if (wasAvailable != renderCallbackAvailable()) emit renderCallbackAvailableChanged();
    update();
}

bool StreamVideoItem::frameGeneration() const
{
    return m_frameGeneration;
}

void StreamVideoItem::setFrameGeneration(bool enabled)
{
    if (m_frameGeneration == enabled) return;
    m_frameGeneration = enabled;
    if (enabled) m_frameStatsTimer.start();
    else m_frameStatsTimer.stop();
    emit frameGenerationChanged();
    emit frameGenerationStatsChanged();
    update();
}

QVariantMap StreamVideoItem::frameGenerationStats() const
{
    return m_renderCallback ? m_renderCallback->frameGenerationStats() : QVariantMap{};
}

QVariantMap StreamVideoItem::swapStats() const
{
    QVariantMap stats = m_renderCallback ? m_renderCallback->swapStats() : QVariantMap{};
    stats.insert(QStringLiteral("gated"), !m_swapGateSource.isEmpty());
    if (!m_swapGateSource.isEmpty())
        stats.insert(QStringLiteral("gateSource"), m_swapGateSource);
    return stats;
}

void StreamVideoItem::writeLiveTelemetry()
{
    if (m_liveTelemetryPath.isEmpty() || m_liveTelemetryFailed) return;
    auto stats = swapStats();
    if (s_nativeRuntime) stats.insert(s_nativeRuntime->frameNotificationStats());
    QJsonObject snapshot{
        {QStringLiteral("event"), QStringLiteral("qt-presentation")},
        {QStringLiteral("observerId"), double(m_liveTelemetryObserverId)},
        {QStringLiteral("wallTimeMs"), double(QDateTime::currentMSecsSinceEpoch())},
        {QStringLiteral("monotonicNs"), double(streamMonotonicClockNs())},
        {QStringLiteral("visible"), isVisible()},
        {QStringLiteral("windowActive"), window() && window()->isActive()},
        {QStringLiteral("fullscreen"), window() && window()->visibility() == QWindow::FullScreen},
        {QStringLiteral("displayHz"), window() && window()->screen() ? window()->screen()->refreshRate() : 0},
        {QStringLiteral("frameGeneration"), m_frameGeneration},
        {QStringLiteral("inputEnabled"), m_inputEnabled},
        {QStringLiteral("captureActive"), m_captureActive},
        {QStringLiteral("runtimeRunning"), s_nativeRuntime && s_nativeRuntime->running()},
        {QStringLiteral("presentationAllowed"), s_nativeRuntime && s_nativeRuntime->presentationAllowed()},
        {QStringLiteral("presentationGeneration"), s_nativeRuntime ? double(s_nativeRuntime->presentationGeneration()) : 0},
        {QStringLiteral("videoWidth"), m_videoSize.width()},
        {QStringLiteral("videoHeight"), m_videoSize.height()},
        {QStringLiteral("gated"), stats.value(QStringLiteral("gated")).toBool()},
        {QStringLiteral("hasPendingSubmit"), stats.value(QStringLiteral("hasPendingSubmit")).toBool()},
        {QStringLiteral("notificationStatsAvailable"), stats.value(QStringLiteral("notificationStatsAvailable")).toBool()},
        {QStringLiteral("lateThresholdMs"), 25},
        {QStringLiteral("sinceLastSourceSwapMs"), QJsonValue::Null},
        {QStringLiteral("relativeMediaLagMs"), QJsonValue::Null},
    };
    const auto number = [&stats, &snapshot](const QString &source, const QString &target) {
        const auto value = stats.value(source);
        switch (value.metaType().id()) {
        case QMetaType::Int:
        case QMetaType::UInt:
        case QMetaType::LongLong:
        case QMetaType::ULongLong:
        case QMetaType::Double:
        case QMetaType::Float: {
            const double numeric = value.toDouble();
            if (std::isfinite(numeric)) snapshot.insert(target, numeric);
            break;
        }
        default: break;
        }
    };
    for (const auto *key : {
             "epoch", "sourceCountersEpoch", "gateEpoch", "swappedFramesTotal", "sourceSwapsTotal",
             "sourceIntervalSamplesTotal", "sourceIntervalOverflowTotal", "lateSourceIntervalsTotal",
             "sourceIntervalMaxMs", "submitSamplesTotal", "submitOverflowTotal", "submitMaxMs",
             "relativeMediaLagMs", "ptsDiscontinuitiesTotal",
             "windowSwapsTotal", "windowSwapsWithoutFreshSourceTotal", "windowIntervalSamplesTotal",
             "windowIntervalOverflowTotal", "windowIntervalMaxMs",
             "notificationTimingEpoch", "notificationEnqueuedTotal", "notificationCoalescedTotal",
             "notificationDeliveredTotal", "notificationDrainSamplesTotal",
             "notificationOldestOverflowTotal", "notificationLatestOverflowTotal",
             "notificationOldestMaxMs", "notificationLatestMaxMs"}) {
        const auto name = QString::fromLatin1(key);
        number(name, name);
    }
    if (stats.value(QStringLiteral("sourceSwapsTotal")).toULongLong() != 0)
        number(QStringLiteral("sinceLastSwapMs"), QStringLiteral("sinceLastSourceSwapMs"));
    for (const auto *key : {"sourceIntervalHistogramMs", "submitHistogramMs",
                            "windowIntervalHistogramMs", "notificationOldestHistogramMs",
                            "notificationLatestHistogramMs"}) {
        const auto name = QString::fromLatin1(key);
        const auto values = stats.value(name).toList();
        if (values.size() != 512) continue;
        QJsonArray histogram;
        bool valid = true;
        for (const auto &value : values) {
            if (value.metaType().id() != QMetaType::ULongLong) {
                valid = false;
                break;
            }
            histogram.append(double(value.toULongLong()));
        }
        if (valid) snapshot.insert(name, histogram);
    }
    QFile file(m_liveTelemetryPath);
    if (QFileInfo(m_liveTelemetryPath).isSymLink()
        || !file.open(QIODevice::WriteOnly | QIODevice::Append)
        || !file.setPermissions(QFileDevice::ReadOwner | QFileDevice::WriteOwner)) {
        m_liveTelemetryFailed = true;
        qWarning("Live presentation telemetry unavailable");
        return;
    }
    const auto line = QJsonDocument(snapshot).toJson(QJsonDocument::Compact) + '\n';
    if (file.write(line) != line.size() || !file.flush()) {
        m_liveTelemetryFailed = true;
        qWarning("Live presentation telemetry write failed");
    }
}

QString StreamVideoItem::currentSwapGateSource() const
{
    if (const auto *currentWindow = window();
               currentWindow
               && (!currentWindow->isVisible()
                   || currentWindow->visibility() == QWindow::Minimized)) {
        return currentWindow->isVisible() ? QStringLiteral("minimized")
                                          : QStringLiteral("hidden");
    }
    if (!isVisible()) return QStringLiteral("hidden");
    return {};
}

void StreamVideoItem::pushSwapGate()
{
    if (m_renderCallback)
        m_renderCallback->setSwapGated(!m_swapGateSource.isEmpty(), m_swapGateSource);
    emit swapStatsChanged();
}

void StreamVideoItem::updateSwapGate()
{
    const auto source = currentSwapGateSource();
    if (source == m_swapGateSource) return;
    m_swapGateSource = source;
    pushSwapGate();
}

void StreamVideoItem::syncSwapGate()
{
    m_swapGateSource = currentSwapGateSource();
    pushSwapGate();
}

int StreamVideoItem::upscalingSharpness() const
{
    return m_upscalingSharpness;
}

void StreamVideoItem::setUpscalingSharpness(int value)
{
    value = qBound(0, value, 15);
    if (m_upscalingSharpness == value) return;
    m_upscalingSharpness = value;
    emit upscalingSharpnessChanged();
    update();
}

int StreamVideoItem::upscalingDenoise() const
{
    return m_upscalingDenoise;
}

void StreamVideoItem::setUpscalingDenoise(int value)
{
    value = qBound(0, value, 20);
    if (m_upscalingDenoise == value) return;
    m_upscalingDenoise = value;
    emit upscalingDenoiseChanged();
    update();
}

bool StreamVideoItem::metalFxUpscaling() const
{
    return m_metalFxUpscaling;
}

void StreamVideoItem::setMetalFxUpscaling(bool enabled)
{
#if !defined(Q_OS_MACOS)
    enabled = false;
#endif
    if (m_metalFxUpscaling == enabled) return;
    m_metalFxUpscaling = enabled;
    emit metalFxUpscalingChanged();
    update();
}

bool StreamVideoItem::fsrUpscaling() const
{
    return m_fsrUpscaling;
}

void StreamVideoItem::setFsrUpscaling(bool enabled)
{
    if (m_fsrUpscaling == enabled) return;
    m_fsrUpscaling = enabled;
    emit fsrUpscalingChanged();
    update();
}

void StreamVideoItem::connectFrameSwaps()
{
    static const bool continuousVideoUpdate =
        qEnvironmentVariable("OPENNOW_CONTINUOUS_VIDEO_UPDATE") == QStringLiteral("1");
    disconnect(m_frameSwapConnection);
    disconnect(m_frameUpdateConnection);
    if (!window() || !m_renderCallback) return;
    const std::weak_ptr<StreamVideoRenderCallback> callback = m_renderCallback;
    m_frameSwapConnection = connect(window(), &QQuickWindow::frameSwapped, this, [callback] {
        if (auto renderer = callback.lock()) renderer->frameSwapped();
    }, Qt::DirectConnection);
    m_frameUpdateConnection = connect(window(), &QQuickWindow::frameSwapped, this, [this] {
        const bool ordinaryVideoUpdate = continuousVideoUpdate && !m_frameGeneration
            && window() && window()->isVisible() && window()->visibility() != QWindow::Minimized
            && isVisible() && m_renderCallback && m_renderCallback->hasVideoFrame()
            && s_nativeRuntime && s_nativeRuntime->running() && s_nativeRuntime->presentationAllowed();
        if (ordinaryVideoUpdate
                || (m_frameGeneration && isVisible() && m_renderCallback && m_renderCallback->needsFrame()))
            update();
    }, Qt::QueuedConnection);
}

void StreamVideoItem::setNativeStreamRuntime(NativeStreamRuntime *runtime)
{
    s_nativeRuntime = runtime;
}

NativeStreamRuntime *StreamVideoItem::nativeStreamRuntime()
{
    return s_nativeRuntime.data();
}

void StreamVideoItem::requestFrame()
{
    if (QThread::currentThread() != thread()) {
        QMetaObject::invokeMethod(this, &StreamVideoItem::requestFrame, Qt::QueuedConnection);
        return;
    }
    update();
}

QRect StreamVideoItem::aspectFitRect(const QSize &videoSize, const QSize &targetSize)
{
    if (!targetSize.isValid() || targetSize.isEmpty()) return {};
    if (!videoSize.isValid() || videoSize.isEmpty()) return QRect(QPoint{}, targetSize);

    const auto scale = std::min(static_cast<double>(targetSize.width()) / videoSize.width(),
                                static_cast<double>(targetSize.height()) / videoSize.height());
    const auto width = std::max(1, static_cast<int>(std::floor(videoSize.width() * scale)));
    const auto height = std::max(1, static_cast<int>(std::floor(videoSize.height() * scale)));
    return QRect((targetSize.width() - width) / 2,
                 (targetSize.height() - height) / 2,
                 width, height);
}

void registerStreamVideoItemQmlType()
{
    qmlRegisterType<StreamVideoItem>("OpenNOW", 1, 0, "StreamVideoItem");
}
