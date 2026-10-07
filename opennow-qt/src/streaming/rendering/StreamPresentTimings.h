#pragma once

#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <mutex>
#include <vector>

class StreamPresentTimings
{
public:
    static constexpr std::size_t WindowCapacity = 256;
    static constexpr std::size_t HistogramCapacity = 512;
    using Histogram = std::array<std::uint64_t, HistogramCapacity>;

    struct Stage
    {
        std::int64_t p50Ns = 0;
        std::int64_t p95Ns = 0;
        std::int64_t maxNs = 0;
    };

    struct Snapshot
    {
        bool available = false;
        Stage submitToSwap;
        std::size_t windowSamples = 0;
        std::uint64_t swappedFramesTotal = 0;
        bool hasLastSwap = false;
        std::int64_t lastSwapNs = 0;
        bool hasPendingSubmit = false;
        std::uint64_t epoch = 0;
        bool gated = false;
        std::uint64_t gateEpoch = 0;
        Histogram sourceIntervalHistogramMs{};
        Histogram submitHistogramMs{};
        std::uint64_t sourceSwapsTotal = 0;
        std::uint64_t sourceIntervalSamplesTotal = 0;
        std::uint64_t sourceIntervalOverflowTotal = 0;
        std::uint64_t lateSourceIntervalsTotal = 0;
        std::int64_t sourceIntervalMaxNs = 0;
        std::uint64_t submitSamplesTotal = 0;
        std::uint64_t submitOverflowTotal = 0;
        std::int64_t submitMaxNs = 0;
        bool hasRelativeMediaLag = false;
        double relativeMediaLagMs = 0;
        std::uint64_t ptsDiscontinuitiesTotal = 0;
        Histogram windowIntervalHistogramMs{};
        std::uint64_t windowSwapsTotal = 0;
        std::uint64_t windowSwapsWithoutFreshSourceTotal = 0;
        std::uint64_t windowIntervalSamplesTotal = 0;
        std::uint64_t windowIntervalOverflowTotal = 0;
        std::int64_t windowIntervalMaxNs = 0;
    };

    void markSubmit(std::int64_t nowNs, std::uint64_t mediaPtsNs = 0)
    {
        const std::lock_guard lock(m_mutex);
        if (m_gated) return;
        m_submitNs = nowNs;
        m_pendingMediaPtsNs = mediaPtsNs;
        m_hasPendingSubmit = true;
    }

    void markSwap(std::int64_t nowNs)
    {
        const std::lock_guard lock(m_mutex);
        if (m_gated || !m_hasPendingSubmit) return;
        m_hasPendingSubmit = false;
        const std::int64_t delta = nowNs > m_submitNs ? nowNs - m_submitNs : 0;
        addHistogram(m_submitHistogramMs, m_submitOverflowTotal, delta);
        ++m_submitSamplesTotal;
        m_submitMaxNs = std::max(m_submitMaxNs, delta);
        if (m_hasIntervalAnchor && nowNs > m_intervalAnchorNs) {
            const auto interval = nowNs - m_intervalAnchorNs;
            addHistogram(m_sourceIntervalHistogramMs, m_sourceIntervalOverflowTotal, interval);
            ++m_sourceIntervalSamplesTotal;
            if (interval > 25'000'000) ++m_lateSourceIntervalsTotal;
            m_sourceIntervalMaxNs = std::max(m_sourceIntervalMaxNs, interval);
        }
        m_intervalAnchorNs = nowNs;
        m_hasIntervalAnchor = true;
        ++m_sourceSwapsTotal;
        if (m_pendingMediaPtsNs != 0) {
            if (!m_hasMediaAnchor || m_pendingMediaPtsNs <= m_previousMediaPtsNs) {
                if (m_hasMediaAnchor) ++m_ptsDiscontinuitiesTotal;
                m_mediaAnchorPtsNs = m_pendingMediaPtsNs;
                m_mediaAnchorSwapNs = nowNs;
                m_hasMediaAnchor = true;
            }
            m_relativeMediaLagMs = (double(nowNs - m_mediaAnchorSwapNs)
                - double(m_pendingMediaPtsNs - m_mediaAnchorPtsNs)) / 1.0e6;
            m_previousMediaPtsNs = m_pendingMediaPtsNs;
        } else {
            m_hasMediaAnchor = false;
        }
        m_samples[m_sampleCount % WindowCapacity] = delta;
        ++m_sampleCount;
        m_windowSamples = std::min(m_sampleCount, WindowCapacity);
        ++m_swappedFramesTotal;
        m_lastSwapNs = nowNs;
        m_hasLastSwap = true;
    }

    void markWindowSwap(std::int64_t nowNs, bool freshSourceSubmit)
    {
        const std::lock_guard lock(m_mutex);
        if (m_gated) return;
        ++m_windowSwapsTotal;
        if (!freshSourceSubmit) ++m_windowSwapsWithoutFreshSourceTotal;
        if (m_hasWindowIntervalAnchor && nowNs > m_windowIntervalAnchorNs) {
            const auto interval = nowNs - m_windowIntervalAnchorNs;
            addHistogram(m_windowIntervalHistogramMs, m_windowIntervalOverflowTotal, interval);
            ++m_windowIntervalSamplesTotal;
            m_windowIntervalMaxNs = std::max(m_windowIntervalMaxNs, interval);
        }
        m_windowIntervalAnchorNs = nowNs;
        m_hasWindowIntervalAnchor = true;
    }

    Snapshot snapshot() const
    {
        const std::lock_guard lock(m_mutex);
        Snapshot result;
        result.windowSamples = m_windowSamples;
        result.swappedFramesTotal = m_swappedFramesTotal;
        result.hasLastSwap = m_hasLastSwap;
        result.lastSwapNs = m_lastSwapNs;
        result.hasPendingSubmit = m_hasPendingSubmit;
        result.epoch = m_epoch;
        result.gated = m_gated;
        result.gateEpoch = m_gateEpoch;
        result.sourceIntervalHistogramMs = m_sourceIntervalHistogramMs;
        result.submitHistogramMs = m_submitHistogramMs;
        result.sourceSwapsTotal = m_sourceSwapsTotal;
        result.sourceIntervalSamplesTotal = m_sourceIntervalSamplesTotal;
        result.sourceIntervalOverflowTotal = m_sourceIntervalOverflowTotal;
        result.lateSourceIntervalsTotal = m_lateSourceIntervalsTotal;
        result.sourceIntervalMaxNs = m_sourceIntervalMaxNs;
        result.submitSamplesTotal = m_submitSamplesTotal;
        result.submitOverflowTotal = m_submitOverflowTotal;
        result.submitMaxNs = m_submitMaxNs;
        result.hasRelativeMediaLag = m_hasMediaAnchor;
        result.relativeMediaLagMs = m_relativeMediaLagMs;
        result.ptsDiscontinuitiesTotal = m_ptsDiscontinuitiesTotal;
        result.windowIntervalHistogramMs = m_windowIntervalHistogramMs;
        result.windowSwapsTotal = m_windowSwapsTotal;
        result.windowSwapsWithoutFreshSourceTotal = m_windowSwapsWithoutFreshSourceTotal;
        result.windowIntervalSamplesTotal = m_windowIntervalSamplesTotal;
        result.windowIntervalOverflowTotal = m_windowIntervalOverflowTotal;
        result.windowIntervalMaxNs = m_windowIntervalMaxNs;
        if (m_windowSamples == 0) return result;
        std::vector<std::int64_t> sorted(m_samples.begin(), m_samples.begin() + m_windowSamples);
        std::sort(sorted.begin(), sorted.end());
        result.available = true;
        result.submitToSwap.p50Ns = percentile(sorted, 50);
        result.submitToSwap.p95Ns = percentile(sorted, 95);
        result.submitToSwap.maxNs = sorted.back();
        return result;
    }

    void setGated(bool gated)
    {
        const std::lock_guard lock(m_mutex);
        if (gated && !m_gated) ++m_gateEpoch;
        m_gated = gated;
        if (gated) {
            m_hasPendingSubmit = false;
            m_hasIntervalAnchor = false;
            m_hasWindowIntervalAnchor = false;
            m_hasMediaAnchor = false;
        }
    }

    void discardPending()
    {
        const std::lock_guard lock(m_mutex);
        m_hasPendingSubmit = false;
    }

    void reset()
    {
        const std::lock_guard lock(m_mutex);
        m_hasPendingSubmit = false;
        m_sampleCount = 0;
        m_windowSamples = 0;
        m_sourceIntervalHistogramMs = {};
        m_submitHistogramMs = {};
        m_sourceSwapsTotal = 0;
        m_sourceIntervalSamplesTotal = 0;
        m_sourceIntervalOverflowTotal = 0;
        m_lateSourceIntervalsTotal = 0;
        m_sourceIntervalMaxNs = 0;
        m_submitSamplesTotal = 0;
        m_submitOverflowTotal = 0;
        m_submitMaxNs = 0;
        m_hasIntervalAnchor = false;
        m_hasMediaAnchor = false;
        m_ptsDiscontinuitiesTotal = 0;
        m_windowIntervalHistogramMs = {};
        m_windowSwapsTotal = 0;
        m_windowSwapsWithoutFreshSourceTotal = 0;
        m_windowIntervalSamplesTotal = 0;
        m_windowIntervalOverflowTotal = 0;
        m_windowIntervalMaxNs = 0;
        m_hasWindowIntervalAnchor = false;
        ++m_epoch;
    }

private:
    static void addHistogram(Histogram &histogram, std::uint64_t &overflow,
                             std::int64_t durationNs)
    {
        const auto milliseconds = std::uint64_t(durationNs / 1'000'000);
        if (milliseconds < HistogramCapacity) ++histogram[milliseconds];
        else ++overflow;
    }

    static std::int64_t percentile(const std::vector<std::int64_t> &sorted, std::size_t percent)
    {
        const std::size_t rank = std::max<std::size_t>(1, (sorted.size() * percent + 99) / 100);
        return sorted[std::min(rank, sorted.size()) - 1];
    }

    mutable std::mutex m_mutex;
    std::vector<std::int64_t> m_samples = std::vector<std::int64_t>(WindowCapacity);
    std::size_t m_sampleCount = 0;
    std::size_t m_windowSamples = 0;
    std::uint64_t m_swappedFramesTotal = 0;
    std::int64_t m_submitNs = 0;
    bool m_hasPendingSubmit = false;
    std::int64_t m_lastSwapNs = 0;
    bool m_hasLastSwap = false;
    bool m_gated = false;
    std::uint64_t m_gateEpoch = 0;
    std::uint64_t m_epoch = 0;
    Histogram m_sourceIntervalHistogramMs{};
    Histogram m_submitHistogramMs{};
    std::uint64_t m_sourceSwapsTotal = 0;
    std::uint64_t m_sourceIntervalSamplesTotal = 0;
    std::uint64_t m_sourceIntervalOverflowTotal = 0;
    std::uint64_t m_lateSourceIntervalsTotal = 0;
    std::int64_t m_sourceIntervalMaxNs = 0;
    std::uint64_t m_submitSamplesTotal = 0;
    std::uint64_t m_submitOverflowTotal = 0;
    std::int64_t m_submitMaxNs = 0;
    bool m_hasIntervalAnchor = false;
    std::int64_t m_intervalAnchorNs = 0;
    std::uint64_t m_pendingMediaPtsNs = 0;
    bool m_hasMediaAnchor = false;
    std::uint64_t m_mediaAnchorPtsNs = 0;
    std::uint64_t m_previousMediaPtsNs = 0;
    std::int64_t m_mediaAnchorSwapNs = 0;
    double m_relativeMediaLagMs = 0;
    std::uint64_t m_ptsDiscontinuitiesTotal = 0;
    Histogram m_windowIntervalHistogramMs{};
    std::uint64_t m_windowSwapsTotal = 0;
    std::uint64_t m_windowSwapsWithoutFreshSourceTotal = 0;
    std::uint64_t m_windowIntervalSamplesTotal = 0;
    std::uint64_t m_windowIntervalOverflowTotal = 0;
    std::int64_t m_windowIntervalMaxNs = 0;
    std::int64_t m_windowIntervalAnchorNs = 0;
    bool m_hasWindowIntervalAnchor = false;
};
