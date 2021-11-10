import zju.measure.DiscreteInfo
import zju.measure.MeasureInfo

/**
 * 提供量测数据的接口
 * @date 17-1-8.
 */
interface MeasureProvider {

    fun getAnalogValue(measId: String, dataSource: String): Double?

    fun getDiscreteValue(measId: String, dataSource: String): Int?

    fun getAnalogsOfEquip(equip: String, dataSource: String): List<MeasureInfo>

    fun getDiscreteOfEquip(equip: String, dataSource: String): DiscreteInfo?

    fun setResourceManager(resourceManager: PowerSystem)

    fun getSwitchStatus(switchId: String, dataSource: String): Boolean
}