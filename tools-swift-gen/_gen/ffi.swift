//
//  globalIndex.swift
//
//  Contains:
//  var globalIndex
//  var globalCallbacks
//  protocol Callback
//  class TestContract1
//  struct StructSimple
//
//  Generated by SwiftPoet on 2019/3/7
//

import rustlib.demo

var globalIndex: Int64 = 0

var globalCallbacks: Dictionary<Int64,Any> = [Int64: Any]()

public protocol Callback {


    /**
        :param:    arg1

        :param:    arg2

        :param:    arg3

        :param:    arg4

        :param:    arg5
    */
    func on_callback(arg1: Int, arg2: String, arg3: Bool, arg4: Double, arg5: Double) -> Int

    /**
        :param:    arg1
    */
    func on_callback2(arg1: Bool) -> Bool

    /**
        :param:    arg1
    */
    func on_callback_complex(arg1: StructSimple) -> Bool

    /**
        :param:    arg1
    */
    func on_callback_arg_vec(arg1: Array<StructSimple>) -> Bool

    /**
        :param:    arg1
    */
    func on_callback_arg_vec_simple(arg1: Array<String>) -> Bool

}

public class TestContract1 {


    /**
        :param:    arg
    */
    public static func test_arg_vec(arg: Array<String>) -> Int {
        
        let encoder = JSONEncoder()
        let data_arg = try! encoder.encode(arg)
        let s_arg = String(data: data_arg, encoding: .utf8)!
        let result = test_contract1_test_arg_vec(s_arg)
        let s_result = Int(result)
        return s_result
    }

    /**
        :param:    arg
    */
    public static func test_return_vec(arg: Int) -> Array<Int> {
        
        let s_arg = Int32(arg)
        let result = test_contract1_test_return_vec(s_arg)
        let ret_str = String(cString:result!)
        let ret_str_json = ret_str.data(using: .utf8)!
        let decoder = JSONDecoder()
        let s_result = try! decoder.decode([Int].self, from: ret_str_json)
        demo_free_str(result!)
        return s_result
    }

    /**
        :param:    arg
    */
    public static func test_arg_callback(arg: Callback) -> Int {
        
        let arg_index = globalIndex + 1
        globalIndex = arg_index
        globalCallbacks[arg_index] = arg
        let arg_on_callback : @convention(c) (Int64, Int32, UnsafePointer<Int8>?, Int32, Float32, Float64) -> Int32 = { 
        
        (index, arg1, arg2, arg3, arg4, arg5) -> Int32 in
        let arg_callback = globalCallbacks[index] as! Callback
        let c_arg1 = Int(arg1)
        let c_arg2 = String(cString: arg2!)
        let c_arg3: Bool = arg3 > 0 ? true : false
        let c_arg4 = Double(arg4)
        let c_arg5 = Double(arg5)
        let result = arg_callback.on_callback(arg1:c_arg1,arg2:c_arg2,arg3:c_arg3,arg4:c_arg4,arg5:c_arg5)
        return Int32(result)
        }
        let arg_on_callback2 : @convention(c) (Int64, Int32) -> Int32 = { 
        
        (index, arg1) -> Int32 in
        let arg_callback = globalCallbacks[index] as! Callback
        let c_arg1: Bool = arg1 > 0 ? true : false
        let result = arg_callback.on_callback2(arg1:c_arg1)
        return result ? 1 : 0
        }
        let arg_on_callback_complex : @convention(c) (Int64, UnsafePointer<Int8>?) -> Int32 = { 
        
        (index, arg1) -> Int32 in
        let arg_callback = globalCallbacks[index] as! Callback
        let c_tmp_arg1 = String(cString:arg1!)
        let c_tmp_json_arg1 = c_tmp_arg1.data(using: .utf8)!
        let decoder = JSONDecoder()
        let c_arg1 = try! decoder.decode(StructSimple.self, from: c_tmp_json_arg1)
        let result = arg_callback.on_callback_complex(arg1:c_arg1)
        return result ? 1 : 0
        }
        let arg_on_callback_arg_vec : @convention(c) (Int64, UnsafePointer<Int8>?) -> Int32 = { 
        
        (index, arg1) -> Int32 in
        let arg_callback = globalCallbacks[index] as! Callback
        let c_tmp_arg1 = String(cString:arg1!)
        let c_tmp_json_arg1 = c_tmp_arg1.data(using: .utf8)!
        let decoder = JSONDecoder()
        let c_arg1 = try! decoder.decode(Array<StructSimple>.self, from: c_tmp_json_arg1)
        let result = arg_callback.on_callback_arg_vec(arg1:c_arg1)
        return result ? 1 : 0
        }
        let arg_on_callback_arg_vec_simple : @convention(c) (Int64, UnsafePointer<Int8>?) -> Int32 = { 
        
        (index, arg1) -> Int32 in
        let arg_callback = globalCallbacks[index] as! Callback
        let c_tmp_arg1 = String(cString:arg1!)
        let c_tmp_json_arg1 = c_tmp_arg1.data(using: .utf8)!
        let decoder = JSONDecoder()
        let c_arg1 = try! decoder.decode([String].self, from: c_tmp_json_arg1)
        let result = arg_callback.on_callback_arg_vec_simple(arg1:c_arg1)
        return result ? 1 : 0
        }
        let callback_free : @convention(c)(Int64) -> () = {
        (index) in
        globalCallbacks.removeValue(forKey: index)
        }
        let s_arg = test_contract1_Callback_Model(on_callback:arg_on_callback,on_callback2:arg_on_callback2,on_callback_complex:arg_on_callback_complex,on_callback_arg_vec:arg_on_callback_arg_vec,on_callback_arg_vec_simple:arg_on_callback_arg_vec_simple,free_callback: callback_free, index: arg_index)
        let result = test_contract1_test_arg_callback(s_arg)
        let s_result = Int(result)
        return s_result
    }

    /**
        :param:    arg1
    */
    public static func test_bool(arg1: Bool) -> Bool {
        
        let s_arg1: Int32 = arg1 ? 1 : 0
        let result = test_contract1_test_bool(s_arg1)
        let s_result = result > 0 ? true : false
        return s_result
    }

    public static func test_struct() -> StructSimple {
        
        let result = test_contract1_test_struct()
        let ret_str = String(cString:result!)
        let ret_str_json = ret_str.data(using: .utf8)!
        let decoder = JSONDecoder()
        let s_result = try! decoder.decode(StructSimple.self, from: ret_str_json)
        demo_free_str(need_free!, result!)
        return s_result
    }

    public static func test_struct_vec() -> Array<StructSimple> {
        
        let result = test_contract1_test_struct_vec()
        let ret_str = String(cString:result!)
        let ret_str_json = ret_str.data(using: .utf8)!
        let decoder = JSONDecoder()
        let s_result = try! decoder.decode(Array<StructSimple>.self, from: ret_str_json)
        demo_free_str(result!)
        return s_result
    }

}

public struct StructSimple: Codable {
    public let arg1: Int
    public let arg2: Int
    public let arg3: String
    public let arg4: Bool
    public let arg5: Double
    public let art6: Double
}
